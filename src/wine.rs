use std::{
    io,
    path::{Path, PathBuf},
    process::{Child, Command},
};

pub fn is_available() -> bool {
    Command::new("wine")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

pub fn version() -> Option<String> {
    let output = Command::new("wine")
        .arg("--version")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    (!version.is_empty()).then_some(version)
}

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            ext.eq_ignore_ascii_case("exe") || ext.eq_ignore_ascii_case("msi")
        })
}

pub fn open(path: &Path) -> io::Result<Child> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File does not exist: {}", path.display()),
        ));
    }

    if !is_supported(path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unsupported Wine file: {}", path.display()),
        ));
    }

    if !is_available() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Wine is not installed or could not be found in PATH",
        ));
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();

    if extension.eq_ignore_ascii_case("msi") {
        Command::new("wine")
            .arg("msiexec")
            .arg("/i")
            .arg(path)
            .spawn()
    } else {
        Command::new("wine")
            .arg(path)
            .spawn()
    }
}

pub fn open_with_prefix(path: &Path, prefix: &Path) -> io::Result<Child> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File does not exist: {}", path.display()),
        ));
    }

    if !prefix.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Wine prefix does not exist: {}", prefix.display()),
        ));
    }

    if !is_supported(path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unsupported Wine file: {}", path.display()),
        ));
    }

    if !is_available() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Wine is not installed or could not be found in PATH",
        ));
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();

    let mut command = Command::new("wine");

    if extension.eq_ignore_ascii_case("msi") {
        command.args(["msiexec", "/i"]);
    }

    command
        .arg(path)
        .env("WINEPREFIX", prefix)
        .spawn()
}

pub fn default_prefix() -> Option<PathBuf> {
    std::env::var_os("WINEPREFIX")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".wine"))
        })
}

pub fn open_default_prefix(path: &Path) -> io::Result<Child> {
    match default_prefix() {
        Some(prefix) => open_with_prefix(path, &prefix),
        None => open(path),
    }
}
