// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Startup fixes for a Finder-launched `.app` bundle.
//!
//! See docs/macos-porting-notes.md section 5.5.

use objc2_foundation::NSLocale;
use std::env;
use std::path::{Path, PathBuf};

/// The `XDG_DATA_DIRS` default from the base directory specification, used when
/// the variable is unset or empty.
const XDG_DATA_DIRS_DEFAULT: &str = "/usr/local/share:/usr/share";

/// The bundle's `Contents/Resources` directory, given the running executable.
///
/// `None` when the executable is not inside `<name>.app/Contents/MacOS/`, which
/// is how a terminal or `cargo run` launch is told apart from a bundle launch.
pub fn bundle_resources(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    let contents = macos.parent()?;
    let app = contents.parent()?;
    if macos.file_name()? == "MacOS"
        && contents.file_name()? == "Contents"
        && app.extension()? == "app"
    {
        Some(contents.join("Resources"))
    } else {
        None
    }
}

/// `XDG_DATA_DIRS` with the bundle's own `share` directory searched first.
///
/// `existing` is the inherited value, which a Finder launch leaves unset; the
/// entries it does carry are kept, so a bundle run from a terminal still sees
/// Homebrew's data. The result is stable under repetition: re-prepending an
/// entry that is already listed does not lengthen the list.
fn data_dirs_with(share: &Path, existing: Option<&str>) -> String {
    let share = share.to_string_lossy();
    let inherited = match existing {
        Some(value) if !value.is_empty() => value,
        _ => XDG_DATA_DIRS_DEFAULT,
    };
    let mut dirs = vec![share.as_ref()];
    dirs.extend(
        inherited
            .split(':')
            .filter(|dir| !dir.is_empty() && *dir != share.as_ref()),
    );
    dirs.join(":")
}

/// A POSIX `LANG` value built from the two `NSLocale` components.
///
/// The notes (5.4) rule out `localeIdentifier`, which carries calendar and
/// currency metadata that is not a locale name. `None` when there is no
/// language, since a bare region says nothing about which catalogue to load.
fn lang_from_components(language: Option<&str>, country: Option<&str>) -> Option<String> {
    let language = language.filter(|code| !code.is_empty())?;
    match country.filter(|code| !code.is_empty()) {
        Some(country) => Some(format!(
            "{}_{}.UTF-8",
            language.to_lowercase(),
            country.to_uppercase()
        )),
        None => Some(format!("{}.UTF-8", language.to_lowercase())),
    }
}

/// The directory the process should move to, or `None` to stay where it is.
///
/// Finder hands a bundle the filesystem root as its working directory, which a
/// file manager would otherwise show as "where you are". Alacritty forces
/// `$HOME` the same way (porting notes 5.4). A terminal launch keeps the cwd
/// the user chose.
fn startup_dir(is_bundle: bool, cwd: &Path, home: &Path) -> Option<PathBuf> {
    (is_bundle && cwd == Path::new("/")).then(|| home.to_path_buf())
}

/// The `LANG` the system settings imply, or `None` if AppKit reports no language.
fn system_lang() -> Option<String> {
    let locale = NSLocale::currentLocale();
    let language = locale.languageCode().to_string();
    // `countryCode` is deprecated in favour of `regionCode`, which only exists
    // from macOS 13; the bundle's LSMinimumSystemVersion is 11.0, where sending
    // `regionCode` would be an unrecognised selector. Keep the older spelling.
    #[allow(deprecated)]
    let country = locale.countryCode().map(|code| code.to_string());
    lang_from_components(Some(&language), country.as_deref())
}

/// Gives a bundle launch the environment a shell would have provided.
///
/// Call this early in `main`: the process is still single-threaded there, so
/// mutating the environment cannot race a reader, and every consumer of these
/// variables — the icon theme, the shared MIME database, the i18n loader —
/// reads its own lazily on first use, which is later.
pub fn prepare() {
    let resources = env::current_exe()
        .ok()
        .as_deref()
        .and_then(bundle_resources);

    if let Some(resources) = &resources {
        let data_dirs = data_dirs_with(
            &resources.join("share"),
            env::var("XDG_DATA_DIRS").ok().as_deref(),
        );
        // SAFETY: single-threaded, as above.
        unsafe { env::set_var("XDG_DATA_DIRS", data_dirs) };
    }

    if env::var_os("LANG").is_none()
        && env::var_os("LC_ALL").is_none()
        && let Some(lang) = system_lang()
    {
        // SAFETY: single-threaded, as above.
        unsafe { env::set_var("LANG", lang) };
    }

    if let Ok(cwd) = env::current_dir()
        && let Some(dir) = startup_dir(resources.is_some(), &cwd, &crate::home_dir())
        && let Err(err) = env::set_current_dir(&dir)
    {
        log::warn!("failed to change directory to {}: {}", dir.display(), err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bundled_executable_resolves_its_resources_directory() {
        assert_eq!(
            bundle_resources(Path::new(
                "/Applications/COSMIC Files.app/Contents/MacOS/cosmic-files"
            )),
            Some(PathBuf::from(
                "/Applications/COSMIC Files.app/Contents/Resources"
            ))
        );
    }

    #[test]
    fn an_unbundled_executable_has_no_resources_directory() {
        assert_eq!(
            bundle_resources(Path::new(
                "/Users/someone/dev/cosmic-files/target/release/cosmic-files"
            )),
            None
        );
        assert_eq!(
            bundle_resources(Path::new("/usr/local/bin/cosmic-files")),
            None
        );
    }

    #[test]
    fn the_bundle_share_directory_is_searched_before_the_inherited_entries() {
        assert_eq!(
            data_dirs_with(
                Path::new("/Applications/COSMIC Files.app/Contents/Resources/share"),
                Some("/opt/homebrew/share:/usr/share")
            ),
            "/Applications/COSMIC Files.app/Contents/Resources/share:/opt/homebrew/share:/usr/share"
        );
    }

    #[test]
    fn an_unset_value_falls_back_to_the_xdg_spec_default() {
        // A Finder launch inherits no XDG_DATA_DIRS at all (porting notes 5.4).
        // Dropping the spec default would take /usr/share away from the app.
        assert_eq!(
            data_dirs_with(Path::new("/A.app/Contents/Resources/share"), None),
            "/A.app/Contents/Resources/share:/usr/local/share:/usr/share"
        );
        assert_eq!(
            data_dirs_with(Path::new("/A.app/Contents/Resources/share"), Some("")),
            "/A.app/Contents/Resources/share:/usr/local/share:/usr/share"
        );
    }

    #[test]
    fn a_share_directory_that_is_already_listed_is_not_repeated() {
        // "Open in new window" spawns the same executable as a child, which
        // inherits what we set here and prepends again. Without this the
        // variable grows a copy per generation.
        let share = Path::new("/A.app/Contents/Resources/share");
        let once = data_dirs_with(share, Some("/usr/share"));
        assert_eq!(
            data_dirs_with(share, Some(&once)),
            "/A.app/Contents/Resources/share:/usr/share"
        );
    }

    #[test]
    fn a_language_and_a_region_make_a_posix_lang_value() {
        assert_eq!(
            lang_from_components(Some("fr"), Some("FR")),
            Some("fr_FR.UTF-8".to_string())
        );
    }

    #[test]
    fn a_language_without_a_region_still_makes_a_lang_value() {
        assert_eq!(
            lang_from_components(Some("fr"), None),
            Some("fr.UTF-8".to_string())
        );
        assert_eq!(
            lang_from_components(Some("fr"), Some("")),
            Some("fr.UTF-8".to_string())
        );
    }

    #[test]
    fn a_region_on_its_own_is_not_a_lang_value() {
        assert_eq!(lang_from_components(None, Some("FR")), None);
        assert_eq!(lang_from_components(Some(""), Some("FR")), None);
    }

    #[test]
    fn a_bundle_started_at_the_filesystem_root_moves_to_the_home_directory() {
        assert_eq!(
            startup_dir(true, Path::new("/"), Path::new("/Users/someone")),
            Some(PathBuf::from("/Users/someone"))
        );
    }

    #[test]
    fn a_terminal_launch_keeps_the_directory_it_was_started_in() {
        assert_eq!(
            startup_dir(false, Path::new("/"), Path::new("/Users/someone")),
            None
        );
        assert_eq!(
            startup_dir(false, Path::new("/tmp"), Path::new("/Users/someone")),
            None
        );
    }

    #[test]
    fn a_bundle_opened_on_a_directory_keeps_that_directory() {
        // `open -a "COSMIC Files" <dir>` and a drop onto the Dock icon both run
        // the bundle with a real working directory; only "/" means Finder.
        assert_eq!(
            startup_dir(
                true,
                Path::new("/Users/someone/dev"),
                Path::new("/Users/someone")
            ),
            None
        );
    }
}
