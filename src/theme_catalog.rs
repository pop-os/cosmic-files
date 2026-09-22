// SPDX-License-Identifier: GPL-3.0-only

//! Discovery of COSMIC theme files, for running outside the COSMIC desktop.
//!
//! libcosmic reads the active theme out of cosmic-config and applies it, but it has no
//! notion of a theme *catalog*: on a COSMIC desktop, cosmic-settings owns that choice and
//! writes the shared stores. Standalone there is nothing to write them, so we find theme
//! files ourselves.
//!
//! A theme is one RON file holding a serialized [`ThemeBuilder`] — the format
//! cosmic-settings imports and exports, and the one the rest of the ecosystem ships — so
//! our files interoperate with theirs. The directories follow the convention already in
//! use: `themes/cosmic` under each XDG data directory. The bundle's own `share` is first
//! on `XDG_DATA_DIRS` (see [`crate::launch_macos`]), so themes shipped inside the app are
//! found by the same pass that finds the user's own.
//!
//! See `docs/cosmic-theme-selection.md` for where these conventions come from.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use cosmic::cosmic_theme::ThemeBuilder;
use cosmic::theme::Theme;

/// A theme file found on the search path.
pub struct NamedTheme {
    /// Display name, and what [`crate::config::AppTheme::Named`] stores. Taken from the
    /// file stem, because a theme file carries no name of its own.
    pub name: String,
    /// Read from the palette rather than guessed from the file name, which is what
    /// cosmic-settings' own import does. Upstream's other scanners infer it from a
    /// `-dark` suffix and so mis-read any theme not named that way.
    pub is_dark: bool,
    theme: Theme,
}

impl NamedTheme {
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

static CATALOG: LazyLock<Vec<NamedTheme>> = LazyLock::new(load);

/// Every theme on the search path, sorted by name. Scanned once per process.
pub fn themes() -> &'static [NamedTheme] {
    &CATALOG
}

/// The theme stored under `name`, if it is still on the search path.
pub fn get(name: &str) -> Option<&'static NamedTheme> {
    CATALOG.iter().find(|theme| theme.name == name)
}

/// `themes/cosmic` under each XDG data directory, most specific first.
fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(local) = dirs::data_local_dir() {
        dirs.push(local.join("themes/cosmic"));
    }
    if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
        dirs.extend(std::env::split_paths(&data_dirs).map(|dir| dir.join("themes/cosmic")));
    }
    dirs
}

fn load() -> Vec<NamedTheme> {
    let dirs = search_dirs();
    let themes = load_from(&dirs);
    // The usual complaint is "my theme isn't in the list", and the answer is almost always
    // which directories were searched, so say both.
    log::info!(
        "found {} themes in {}",
        themes.len(),
        dirs.iter()
            .map(|dir| dir.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    themes
}

fn load_from(dirs: &[PathBuf]) -> Vec<NamedTheme> {
    // Keyed by name so an earlier directory shadows a later one, and so the result comes
    // out sorted for display.
    let mut found = BTreeMap::new();
    for dir in dirs {
        // A missing directory is the normal case, not a failure worth reporting.
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension() != Some(OsStr::new("ron")) {
                continue;
            }
            let Some(name) = path.file_stem().and_then(OsStr::to_str) else {
                continue;
            };
            if found.contains_key(name) {
                continue;
            }
            match read_theme(&path) {
                Ok((is_dark, theme)) => {
                    found.insert(
                        name.to_string(),
                        NamedTheme {
                            name: name.to_string(),
                            is_dark,
                            theme,
                        },
                    );
                }
                // cosmic-settings drops these silently, leaving a theme the user installed
                // and cannot see. Say something instead.
                Err(err) => log::warn!("failed to load theme {}: {}", path.display(), err),
            }
        }
    }
    found.into_values().collect()
}

fn read_theme(path: &Path) -> Result<(bool, Theme), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)?;
    let builder: ThemeBuilder = ron::de::from_str(&text)?;
    let is_dark = builder.palette.is_dark();
    Ok((is_dark, Theme::custom(Arc::new(builder.build()))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vendored_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("res/themes/cosmic")
    }

    /// The themes vendored under `res/themes/cosmic` must all parse. They are written in
    /// an older `ThemeBuilder` shape than the one we build against (`is_frosted` became
    /// `frosted`), so this is really a check that serde's defaulting absorbs the drift.
    #[test]
    fn vendored_themes_parse() {
        let dir = vendored_dir();
        let mut count = 0;
        for entry in std::fs::read_dir(&dir).expect("res/themes/cosmic is missing") {
            let path = entry.expect("unreadable entry").path();
            if path.extension() != Some(OsStr::new("ron")) {
                continue;
            }
            read_theme(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
            count += 1;
        }
        assert_eq!(count, 4, "expected the four vendored themes");
    }

    /// Scanning a directory of theme files yields them all, named by file stem and sorted.
    /// This is what the bundle relies on: `res/themes/cosmic` is copied verbatim to
    /// `Contents/Resources/share/themes/cosmic`, which is on `XDG_DATA_DIRS`.
    #[test]
    fn discovery_names_themes_by_file_stem() {
        let found = load_from(&[vendored_dir()]);
        let names: Vec<&str> = found.iter().map(|theme| theme.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Forest Dark",
                "HP Dev One",
                "System76 Dark",
                "System76 Light"
            ],
            "names come from the file stem, sorted, spaces and all"
        );
    }

    /// An earlier directory shadows a later one, so a user's own copy of a theme wins over
    /// the one shipped in the bundle.
    #[test]
    fn earlier_directories_shadow_later_ones() {
        let missing = Path::new("/nonexistent/themes/cosmic").to_path_buf();
        // A directory that does not exist is skipped rather than ending the scan.
        let found = load_from(&[missing, vendored_dir()]);
        assert_eq!(found.len(), 4);

        // The same directory twice must not produce duplicates.
        let twice = load_from(&[vendored_dir(), vendored_dir()]);
        assert_eq!(twice.len(), 4, "a name seen already is not added again");
    }

    /// Dark and light must come from the palette, not the file name: none of the vendored
    /// themes uses the `-dark` suffix that upstream's scanners key off.
    #[test]
    fn is_dark_comes_from_the_palette() {
        let dir = vendored_dir();
        let (is_dark, _) = read_theme(&dir.join("System76 Light.ron")).expect("light theme");
        assert!(!is_dark, "System76 Light has a light palette");
        let (is_dark, _) = read_theme(&dir.join("Forest Dark.ron")).expect("dark theme");
        assert!(is_dark, "Forest Dark has a dark palette");
    }
}
