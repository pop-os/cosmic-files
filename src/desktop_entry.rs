use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub fn create_desktop_entry(executable_path: &Path) -> io::Result<()> {
    let file_name = executable_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid file name"))?;

    let applications_dir = dirs::data_local_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "could not find data directory"))?
        .join("applications");

    fs::create_dir_all(&applications_dir)?;

    let desktop_file_path = applications_dir.join(format!("{}.desktop", file_name));

    let icon_path = find_icon_for_executable(executable_path);

    let content = format!(
        "[Desktop Entry]\n\
         Version=1.0\n\
         Type=Application\n\
         Name={name}\n\
         Exec={exec}\n\
         {icon}\
         Terminal=false\n\
         StartupNotify=true\n",
        name = file_name,
        exec = executable_path.display(),
        icon = icon_path
            .map(|p| format!("Icon={}\n", p.display()))
            .unwrap_or_default(),
    );

    let mut file = fs::File::create(&desktop_file_path)?;
    file.write_all(content.as_bytes())?;

    update_desktop_database(&applications_dir);

    Ok(())
}

fn find_icon_for_executable(executable_path: &Path) -> Option<std::path::PathBuf> {
    let parent = executable_path.parent()?;
    let stem = executable_path.file_stem()?.to_str()?;

    for ext in &["svg", "png", "xpm"] {
        let icon_path = parent.join(format!("{}.{}", stem, ext));
        if icon_path.exists() {
            return Some(icon_path);
        }
    }

    for dir in &["", "icons", "../icons", "../share/icons"] {
        let base = if dir.is_empty() {
            parent.to_path_buf()
        } else {
            parent.join(dir)
        };

        for ext in &["svg", "png", "xpm"] {
            let icon_path = base.join(format!("{}.{}", stem, ext));
            if icon_path.exists() {
                return Some(icon_path);
            }
        }
    }

    None
}

fn update_desktop_database(applications_dir: &Path) {
    if let Err(err) = std::process::Command::new("update-desktop-database")
        .arg(applications_dir)
        .output()
    {
        log::debug!("update-desktop-database not available: {}", err);
    }
}
