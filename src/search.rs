// SPDX-License-Identifier: GPL-3.0-only

//! Search query compilation, traversal, and structured filtering.
//! Unix filenames are matched as bytes, so non-UTF-8 entries are not skipped.

use mime_guess::{Mime, mime};
use regex::{Regex, RegexBuilder, bytes};
use std::collections::{HashMap, HashSet};
#[cfg(not(feature = "gvfs"))]
use std::env;
use std::fmt;
use std::fs::{self, File, Metadata};
use std::io::Read;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use trash::{TrashItem, TrashItemMetadata};

use crate::mime_icon::{mime_for_name, mime_for_path};
use crate::trash::{Trash, TrashExt};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SearchLocation {
    Path(PathBuf),
    Recents,
    Trash,
}

impl fmt::Display for SearchLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => write!(f, "{}", path.display()),
            Self::Recents => write!(f, "recents"),
            Self::Trash => write!(f, "trash"),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SearchFilter {
    pub file_types: SearchFileTypes,
    pub custom_file_types: Arc<[Mime]>,
    pub date: Option<SearchDate>,
    pub text_matching: SearchTextMatching,
    pub recursive: bool,
    pub raw_regex: bool,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            file_types: SearchFileTypes::default(),
            custom_file_types: Arc::default(),
            date: None,
            text_matching: SearchTextMatching::default(),
            recursive: true,
            raw_regex: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SearchFileType {
    Text,
    Audio,
    Documents,
    Folders,
    Images,
    Pdf,
    Spreadsheets,
    Videos,
}

impl SearchFileType {
    pub const ALL: [Self; 8] = [
        Self::Text,
        Self::Audio,
        Self::Documents,
        Self::Folders,
        Self::Images,
        Self::Pdf,
        Self::Spreadsheets,
        Self::Videos,
    ];
    const fn bit(self) -> u16 {
        1 << self as u16
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SearchFileTypes(u16);
impl SearchFileTypes {
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub const fn contains(self, ty: SearchFileType) -> bool {
        self.0 & ty.bit() != 0
    }
    pub fn toggle(&mut self, ty: SearchFileType) {
        self.0 ^= ty.bit();
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SearchDate {
    Today,
    Yesterday,
    PastWeek,
    PastMonth,
    PastYear,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SearchTextMatching {
    #[default]
    ContentAndFilename,
    FilenameOnly,
}

#[derive(Clone, Debug)]
pub enum SearchItem {
    Path(PathBuf, String, fs::Metadata),
    Trash(TrashItem, TrashItemMetadata),
}

#[derive(Clone, Debug)]
pub struct MimeCandidate {
    pub mime: Mime,
    pub description: String,
    pub search_key: String,
    pub count: u64,
}

/// Maps MIME aliases used by extension-only fallback databases to the
/// canonical type from the system shared MIME database.
pub fn canonical_mime(mime: Mime) -> Mime {
    let Some(extension) =
        mime_guess::get_mime_extensions(&mime).and_then(|extensions| extensions.first())
    else {
        return mime;
    };
    mime_for_name(PathBuf::from(format!("file.{extension}"))).unwrap_or(mime)
}

/// Lists MIME types known to the freedesktop shared MIME database. Types found
/// in the current directory are ranked first by their actual frequency.
pub fn mime_type_candidates(root: Option<PathBuf>, excluded: Arc<[Mime]>) -> Vec<MimeCandidate> {
    let excluded: HashSet<Mime> = excluded.iter().cloned().collect();
    let mut counts = HashMap::<Mime, u64>::new();
    if let Some(root) = root
        && let Ok(entries) = fs::read_dir(root)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata()
                && metadata.is_file()
                && let Some(mime) = mime_for_name(&path)
            {
                *counts
                    // Candidate ranking only needs a stable type estimate. Avoid
                    // opening every file and running the shared MIME magic
                    // database just to populate this dialog.
                    .entry(mime)
                    .or_default() += 1;
            }
        }
    }

    let mut known = HashSet::<Mime>::new();
    #[cfg(feature = "gvfs")]
    known.extend(
        gio::content_types_get_registered()
            .into_iter()
            .filter_map(|content_type| content_type.parse().ok()),
    );
    #[cfg(not(feature = "gvfs"))]
    {
        if let Some(data_home) = dirs::data_dir() {
            read_known_mimes(&data_home.join("mime/types"), &mut known);
        }
        let data_dirs = env::var_os("XDG_DATA_DIRS")
            .map(|value| env::split_paths(&value).collect::<Vec<_>>())
            .unwrap_or_else(|| {
                vec![
                    PathBuf::from("/usr/local/share"),
                    PathBuf::from("/usr/share"),
                ]
            });
        for directory in data_dirs {
            read_known_mimes(&directory.join("mime/types"), &mut known);
        }
    }
    known.extend(counts.keys().cloned());
    known.retain(|mime| {
        !excluded.contains(mime)
            && *mime != "application/pdf"
            // This is the generic "unknown bytes" fallback, not a useful
            // user-selectable file type.
            && *mime != mime::APPLICATION_OCTET_STREAM
    });

    let mut candidates = known
        .into_iter()
        .map(|mime| {
            let description = crate::mime_icon::mime_type_description(&mime, false);
            let search_key = format!("{} {}", description.to_lowercase(), mime.essence_str());
            MimeCandidate {
                count: counts.get(&mime).copied().unwrap_or_default(),
                description,
                search_key,
                mime,
            }
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            // Descriptions are already localized display strings. Avoid
            // allocating lowercase copies O(n log n) times while sorting.
            .then_with(|| a.description.cmp(&b.description))
            .then_with(|| a.mime.essence_str().cmp(b.mime.essence_str()))
    });
    candidates
}

#[cfg(not(feature = "gvfs"))]
fn read_known_mimes(path: &Path, output: &mut HashSet<Mime>) {
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    output.extend(contents.lines().filter_map(|line| line.trim().parse().ok()));
}

enum FilenamePattern {
    Glob(globset::GlobMatcher),
    Regex(bytes::Regex),
}

struct SearchPattern {
    filename: FilenamePattern,
    text: Regex,
}
impl SearchPattern {
    fn compile(term: &str, raw: bool) -> Result<Self, String> {
        let (filename, text_pattern) = if raw {
            (
                FilenamePattern::Regex(
                    bytes::RegexBuilder::new(term)
                        .case_insensitive(true)
                        .build()
                        .map_err(|err| err.to_string())?,
                ),
                term.to_owned(),
            )
        } else {
            let glob = globset::GlobBuilder::new(term)
                .case_insensitive(true)
                .build()
                .map_err(|err| err.to_string())?;
            (
                FilenamePattern::Glob(glob.compile_matcher()),
                regex::escape(term),
            )
        };
        Ok(Self {
            filename,
            text: RegexBuilder::new(&text_pattern)
                .case_insensitive(true)
                .build()
                .map_err(|err| err.to_string())?,
        })
    }
    fn filename_matches(&self, name: &std::ffi::OsStr) -> bool {
        match &self.filename {
            FilenamePattern::Glob(pattern) => pattern.is_match(Path::new(name)),
            FilenamePattern::Regex(pattern) => {
                #[cfg(unix)]
                {
                    pattern.is_match(name.as_bytes())
                }
                #[cfg(not(unix))]
                {
                    pattern.is_match(name.to_string_lossy().as_bytes())
                }
            }
        }
    }
}

pub fn scan_search<F: Fn(SearchItem) -> bool + Sync>(
    location: &SearchLocation,
    term: &str,
    show_hidden: bool,
    filter: SearchFilter,
    callback: F,
) {
    if term.is_empty() {
        return;
    }
    let pattern = match SearchPattern::compile(term, filter.raw_regex) {
        Ok(pattern) => pattern,
        Err(err) => {
            log::debug!("invalid search pattern {term:?}: {err}");
            return;
        }
    };
    match location {
        SearchLocation::Path(root) => scan_path(root, show_hidden, &filter, &pattern, callback),
        SearchLocation::Recents => scan_recents(&filter, &pattern, callback),
        SearchLocation::Trash => Trash::scan_search(callback, &pattern.text),
    }
}

fn scan_path<F: Fn(SearchItem) -> bool + Sync>(
    root: &Path,
    show_hidden: bool,
    filter: &SearchFilter,
    pattern: &SearchPattern,
    callback: F,
) {
    let recursive = filter.recursive && !filter.raw_regex;
    ignore::WalkBuilder::new(root)
        .standard_filters(false)
        .hidden(!show_hidden)
        .max_depth((!recursive).then_some(1))
        .same_file_system(true)
        .build_parallel()
        .run(|| {
            Box::new(|entry_res| {
                let Ok(entry) = entry_res else {
                    return ignore::WalkState::Skip;
                };
                let path = entry.path();
                if path == root {
                    return ignore::WalkState::Continue;
                }
                let metadata = match entry.metadata() {
                    Ok(metadata) => metadata,
                    Err(err) => {
                        log::warn!("failed to read metadata for {}: {err}", path.display());
                        return ignore::WalkState::Continue;
                    }
                };
                if matches_filter(path, entry.file_name(), &metadata, pattern, filter)
                    && !callback(SearchItem::Path(
                        path.to_path_buf(),
                        entry.file_name().to_string_lossy().into_owned(),
                        metadata,
                    ))
                {
                    return ignore::WalkState::Quit;
                }
                ignore::WalkState::Continue
            })
        });
}

fn scan_recents<F: Fn(SearchItem) -> bool + Sync>(
    filter: &SearchFilter,
    pattern: &SearchPattern,
    callback: F,
) {
    let files = match recently_used_xbel::parse_file() {
        Ok(files) => files,
        Err(err) => {
            log::warn!("error reading recent files: {err:?}");
            return;
        }
    };
    for bookmark in files.bookmarks {
        let Some(path) = uri_to_path(bookmark.href).filter(|p| p.exists()) else {
            continue;
        };
        let Some(name) = path.file_name() else {
            continue;
        };
        match path.metadata() {
            Ok(metadata) if matches_filter(&path, name, &metadata, pattern, filter) => {
                if !callback(SearchItem::Path(
                    path.clone(),
                    name.to_string_lossy().into_owned(),
                    metadata,
                )) {
                    break;
                }
            }
            Ok(_) => {}
            Err(err) => log::warn!("failed to read metadata for {}: {err}", path.display()),
        }
    }
}

const CONTENT_MAX_BYTES: u64 = 4 * 1024 * 1024;
fn matches_filter(
    path: &Path,
    name: &std::ffi::OsStr,
    metadata: &Metadata,
    pattern: &SearchPattern,
    filter: &SearchFilter,
) -> bool {
    if let Some(date) = filter.date
        && !matches_date(metadata, date)
    {
        return false;
    }
    let filename_matches = pattern.filename_matches(name);
    let types_active = !filter.raw_regex
        && (!filter.file_types.is_empty() || !filter.custom_file_types.is_empty());
    let needs_mime = types_active
        || (!filename_matches
            && filter.text_matching == SearchTextMatching::ContentAndFilename
            && metadata.is_file());
    let mime = needs_mime.then(|| mime_for_path(path, Some(metadata), false));
    if types_active {
        let mime = mime.as_ref().expect("MIME requested for filtering");
        let base = SearchFileType::ALL
            .into_iter()
            .any(|ty| filter.file_types.contains(ty) && matches_file_type(metadata, mime, ty));
        let custom = metadata.is_file()
            && filter.custom_file_types.iter().any(|ty| {
                crate::mime_icon::mime_types_equal(mime, ty)
                    || crate::mime_icon::is_mime_subclass_of(mime, ty)
            });
        if !base && !custom {
            return false;
        }
    }
    filename_matches
        || (filter.text_matching == SearchTextMatching::ContentAndFilename
            && metadata.is_file()
            && metadata.len() <= CONTENT_MAX_BYTES
            && mime.as_ref().is_some_and(is_text_searchable)
            && File::open(path)
                .ok()
                .and_then(|file| {
                    let mut contents = String::new();
                    file.take(CONTENT_MAX_BYTES + 1)
                        .read_to_string(&mut contents)
                        .ok()
                        .map(|_| contents)
                })
                .is_some_and(|contents| pattern.text.is_match(&contents)))
}

fn matches_date(metadata: &Metadata, date: SearchDate) -> bool {
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    let Ok(zoned) = jiff::Zoned::try_from(modified) else {
        return false;
    };
    let now = jiff::Zoned::now();
    match date {
        SearchDate::Today => zoned.date() == now.date(),
        SearchDate::Yesterday => now.date().yesterday().is_ok_and(|d| zoned.date() == d),
        SearchDate::PastWeek => modified >= SystemTime::now() - Duration::from_secs(7 * 86_400),
        SearchDate::PastMonth => modified >= SystemTime::now() - Duration::from_secs(30 * 86_400),
        SearchDate::PastYear => modified >= SystemTime::now() - Duration::from_secs(365 * 86_400),
    }
}

fn is_text_searchable(mime: &Mime) -> bool {
    mime.type_() == mime::TEXT
        || matches!(
            mime.essence_str(),
            "application/json" | "application/xml" | "application/x-yaml"
        )
}

fn matches_file_type(metadata: &Metadata, mime: &Mime, ty: SearchFileType) -> bool {
    if ty == SearchFileType::Folders {
        return metadata.is_dir();
    }
    if metadata.is_dir() {
        return false;
    }
    match ty {
        SearchFileType::Text => mime.type_() == mime::TEXT,
        SearchFileType::Audio => mime.type_() == mime::AUDIO,
        SearchFileType::Images => mime.type_() == mime::IMAGE,
        SearchFileType::Pdf => *mime == "application/pdf",
        SearchFileType::Videos => mime.type_() == mime::VIDEO,
        SearchFileType::Documents => office_type(mime, "x-office-document"),
        SearchFileType::Spreadsheets => office_type(mime, "x-office-spreadsheet"),
        SearchFileType::Folders => false,
    }
}

fn office_type(mime: &Mime, generic: &str) -> bool {
    #[cfg(feature = "gvfs")]
    if gio::content_type_get_generic_icon_name(mime.essence_str()).as_deref() == Some(generic) {
        return true;
    }
    match generic {
        "x-office-document" => matches!(
            mime.essence_str(),
            "application/msword"
                | "application/rtf"
                | "application/vnd.oasis.opendocument.text"
                | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ),
        "x-office-spreadsheet" => matches!(
            mime.essence_str(),
            "application/vnd.ms-excel"
                | "application/vnd.oasis.opendocument.spreadsheet"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                | "text/csv"
        ),
        _ => false,
    }
}

pub(crate) fn uri_to_path(uri: String) -> Option<PathBuf> {
    uri.parse::<url::Url>().ok().and_then(|url| {
        (url.scheme() == "file")
            .then(|| url.to_file_path().ok())
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    #[test]
    fn normal_uses_glob_syntax() {
        let p = SearchPattern::compile("*.{rs,toml}", false).unwrap();
        assert!(p.filename_matches(OsStr::new("main.rs")));
        assert!(p.filename_matches(OsStr::new("Cargo.toml")));
        assert!(!p.filename_matches(OsStr::new("main.py")));
    }

    #[test]
    fn path_search_does_not_return_its_root() {
        let temp = tempfile::TempDir::new().unwrap();
        fs::write(temp.path().join("child.txt"), b"").unwrap();
        let found = std::sync::Mutex::new(Vec::new());
        scan_search(
            &SearchLocation::Path(temp.path().to_path_buf()),
            "*",
            true,
            SearchFilter::default(),
            |item| {
                if let SearchItem::Path(path, ..) = item {
                    found.lock().unwrap().push(path);
                }
                true
            },
        );
        let found = found.into_inner().unwrap();
        assert_eq!(found, vec![temp.path().join("child.txt")]);
    }
    #[test]
    fn raw_uses_regex() {
        let p = SearchPattern::compile(r"^file-[0-9]+\.rs$", true).unwrap();
        assert!(p.filename_matches(OsStr::new("file-42.rs")));
    }

    #[cfg(unix)]
    #[test]
    fn search_does_not_skip_non_utf8_filenames() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let temp = tempfile::TempDir::new().unwrap();
        let name = OsString::from_vec(b"match-\xff.txt".to_vec());
        fs::write(temp.path().join(name), b"data").unwrap();
        let found = AtomicUsize::new(0);
        scan_search(
            &SearchLocation::Path(temp.path().to_path_buf()),
            "match-*",
            true,
            SearchFilter {
                text_matching: SearchTextMatching::FilenameOnly,
                ..SearchFilter::default()
            },
            |_| {
                found.fetch_add(1, Ordering::Relaxed);
                true
            },
        );
        assert_eq!(found.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn mime_candidate_counts_are_not_recursive() {
        let temp = tempfile::TempDir::new().unwrap();
        fs::write(temp.path().join("one.rs"), b"").unwrap();
        fs::write(temp.path().join("two.rs"), b"").unwrap();
        fs::create_dir(temp.path().join("nested")).unwrap();
        fs::write(temp.path().join("nested/three.rs"), b"").unwrap();

        let expected = mime_for_name("file.rs").expect("Rust MIME type");
        let candidates = mime_type_candidates(Some(temp.path().to_path_buf()), Arc::default());
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.mime != mime::APPLICATION_OCTET_STREAM)
        );
        assert_eq!(
            candidates
                .iter()
                .find(|candidate| candidate.mime == expected)
                .map(|candidate| candidate.count),
            Some(2)
        );
    }

    #[cfg(unix)]
    #[test]
    fn legacy_extension_mimes_are_canonicalized() {
        assert_eq!(
            canonical_mime("text/x-rust".parse().unwrap()),
            "text/rust".parse::<Mime>().unwrap()
        );
        assert_eq!(
            canonical_mime("text/x-toml".parse().unwrap()),
            "application/toml".parse::<Mime>().unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn canonical_rust_and_toml_filters_match_real_files() {
        let temp = tempfile::TempDir::new().unwrap();
        fs::write(temp.path().join("main.rs"), b"fn main() {}\n").unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            b"[package]\nname = \"demo\"\n",
        )
        .unwrap();

        for (pattern, mime) in [("*.rs", "text/rust"), ("*.toml", "application/toml")] {
            let found = std::sync::atomic::AtomicUsize::new(0);
            scan_search(
                &SearchLocation::Path(temp.path().to_path_buf()),
                pattern,
                true,
                SearchFilter {
                    custom_file_types: vec![mime.parse().unwrap()].into(),
                    text_matching: SearchTextMatching::FilenameOnly,
                    ..SearchFilter::default()
                },
                |_| {
                    found.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    true
                },
            );
            assert_eq!(
                found.load(std::sync::atomic::Ordering::Relaxed),
                1,
                "custom filter {mime} did not match {pattern}"
            );
        }
    }
}
