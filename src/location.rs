use std::path::{self, PathBuf};
use std::time::Instant;
use std::{fmt, fs};

use crate::config::{DesktopConfig, IconSizes};
use crate::fl;
use crate::item::{Item, folder_name, item_from_path};
use crate::mounter::MOUNTERS;
use crate::scan::{scan_desktop, scan_network, scan_path, scan_recents};
use crate::trash::{Trash, TrashExt};

#[derive(Clone, Debug)]
pub struct EditLocation {
    pub location: Location,
    pub completions: Option<Vec<(String, PathBuf)>>,
    pub selected: Option<usize>,
}

impl EditLocation {
    pub fn resolve(&self) -> Option<Location> {
        if let Location::Network(uri, ..) = &self.location {
            MOUNTERS
                .values()
                .find_map(|mounter| mounter.dir_info(uri))
                .map(|(uri, display_name, path_opt)| Location::Network(uri, display_name, path_opt))
        } else {
            let Some(selected) = self.selected else {
                return Some(self.location.clone());
            };
            let completions = self.completions.as_ref()?;
            let completion = completions.get(selected)?;
            Some(self.location.with_path(completion.1.clone()).normalize())
        }
    }

    pub fn select(&mut self, forwards: bool) {
        if let Some(completions) = &self.completions {
            if completions.is_empty() {
                self.selected = None;
            } else {
                let mut selected = if forwards {
                    self.selected.and_then(|x| x.checked_add(1)).unwrap_or(0)
                } else {
                    self.selected
                        .and_then(|x| x.checked_sub(1))
                        .unwrap_or(completions.len() - 1)
                };
                if selected >= completions.len() {
                    selected = 0;
                }
                self.selected = Some(selected);

                // Automatically resolve if there is only one completion
                if completions.len() == 1
                    && let Some(resolved) = self.resolve()
                {
                    self.location = resolved;
                    self.selected = None;
                }
            }
        } else {
            self.selected = None;
        }
    }
}

impl From<Location> for EditLocation {
    fn from(location: Location) -> Self {
        Self {
            location,
            completions: None,
            selected: None,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Location {
    Desktop(PathBuf, String, DesktopConfig),
    Network(String, String, Option<PathBuf>),
    Path(PathBuf),
    Recents,
    Search(SearchLocation, String, bool, Instant),
    Trash,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Desktop(path, display, ..) => {
                write!(f, "{} on display {display}", path.display())
            }
            Self::Network(uri, ..) => write!(f, "{uri}"),
            Self::Path(path) => write!(f, "{}", path.display()),
            Self::Recents => write!(f, "recents"),
            Self::Search(location, term, ..) => {
                write!(f, "search {} for {}", location, term)
            }
            Self::Trash => write!(f, "trash"),
        }
    }
}

impl Location {
    pub fn normalize(&self) -> Self {
        if let Location::Network(uri, ..) = self {
            if !uri.ends_with('/') {
                let mut uri = uri.clone();
                uri.push('/');
                self.with_uri(uri)
            } else {
                self.clone()
            }
        } else if let Some(mut path) = self.path_opt().cloned() {
            // Canonicalize path, if possible
            if let Ok(canonical) = fs::canonicalize(&path) {
                path = canonical;
            }
            // Add trailing slash if location is a directory
            if path.is_dir() {
                path.push("");
            }
            self.with_path(path)
        } else {
            self.clone()
        }
    }

    pub fn ancestors(&self) -> Vec<(Self, String)> {
        self.path_opt().map_or_else(Default::default, |path| {
            path.ancestors()
                .scan(false, |found_home, ancestor| {
                    (!*found_home).then(|| {
                        let (name, is_home) = folder_name(ancestor);
                        *found_home = is_home;
                        (self.with_path(ancestor.to_path_buf()), name)
                    })
                })
                .collect()
        })
    }

    pub const fn path_opt(&self) -> Option<&PathBuf> {
        match self {
            Self::Desktop(path, ..) => Some(path),
            Self::Path(path) => Some(path),
            Self::Search(SearchLocation::Path(path), ..) => Some(path),
            Self::Network(_, _, path) => path.as_ref(),
            _ => None,
        }
    }

    pub(crate) fn into_path_opt(self) -> Option<PathBuf> {
        match self {
            Self::Desktop(path, ..) => Some(path),
            Self::Path(path) => Some(path),
            Self::Search(SearchLocation::Path(path), ..) => Some(path),
            Self::Network(_, _, path) => path,
            _ => None,
        }
    }

    pub fn with_path(&self, path: PathBuf) -> Self {
        let path = Self::expand_tilde(path);
        match self {
            Self::Desktop(_, display, desktop_config) => {
                Self::Desktop(path, display.clone(), *desktop_config)
            }
            Self::Path(..) => Self::Path(path),
            Self::Search(SearchLocation::Path(_), term, show_hidden, time) => Self::Search(
                SearchLocation::Path(path),
                term.clone(),
                *show_hidden,
                *time,
            ),

            other => other.clone(),
        }
    }

    pub fn with_uri(&self, uri: String) -> Self {
        if let Self::Network(_, name, path) = self {
            Self::Network(uri, name.clone(), path.clone())
        } else {
            self.clone()
        }
    }

    pub fn scan(&self, sizes: IconSizes) -> (Option<Box<Item>>, Vec<Item>) {
        let items = match self {
            Self::Desktop(path, display, desktop_config) => {
                scan_desktop(path, display, *desktop_config, sizes)
            }
            Self::Path(path) => scan_path(path, sizes),
            Self::Search(..) => {
                // Search is done incrementally
                Vec::new()
            }
            Self::Trash => Trash::scan(sizes),
            Self::Recents => scan_recents(sizes),
            Self::Network(uri, _, _) => scan_network(uri, sizes),
        };
        let parent_item_opt = match self.path_opt() {
            Some(path) => match item_from_path(path, sizes) {
                Ok(item) => Some(Box::new(item)),
                Err(err) => {
                    log::warn!("failed to get item for {}: {}", path.display(), err);
                    None
                }
            },
            //TODO: support other locations?
            None => None,
        };
        (parent_item_opt, items)
    }

    pub fn title(&self) -> String {
        match self {
            Self::Desktop(path, _, _) => {
                let (name, _) = folder_name(path);
                name
            }
            Self::Path(path) => {
                let (name, _) = folder_name(path);
                name
            }
            Self::Search(location, term, ..) => {
                let name = match location {
                    SearchLocation::Path(path) => folder_name(path).0,
                    SearchLocation::Trash => fl!("trash"),
                    SearchLocation::Recents => fl!("recents"),
                };

                //TODO: translate
                format!("Search \"{term}\": {name}")
            }
            Self::Trash => {
                fl!("trash")
            }
            Self::Recents => {
                fl!("recents")
            }
            Self::Network(display_name, ..) => display_name.clone(),
        }
    }

    /// Expand a path that starts with "~" with the
    /// user's home directory
    pub fn expand_tilde(path: PathBuf) -> PathBuf {
        let mut components = path.components();
        match components.next() {
            Some(path::Component::Normal(os_str)) if os_str == "~" => {
                if let Some(home) = dirs::home_dir() {
                    home.join(components.as_path())
                } else {
                    path
                }
            }
            _ => path,
        }
    }

    pub fn is_trash(&self) -> bool {
        matches!(
            self,
            Location::Trash | Location::Search(SearchLocation::Trash, ..)
        )
    }

    pub fn is_recents(&self) -> bool {
        matches!(
            self,
            Location::Recents | Location::Search(SearchLocation::Recents, ..)
        )
    }

    /// Returns true if this location supports paste operations (not Trash)
    pub fn supports_paste(&self) -> bool {
        matches!(
            self,
            Self::Desktop(..)
                | Self::Path(..)
                | Self::Search(..)
                | Self::Recents
                | Self::Network(_, _, Some(_))
        )
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SearchLocation {
    Path(PathBuf),
    Recents,
    Trash,
}

impl std::fmt::Display for SearchLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => write!(f, "{}", path.display()),
            Self::Recents => write!(f, "recents"),
            Self::Trash => write!(f, "trash"),
        }
    }
}
