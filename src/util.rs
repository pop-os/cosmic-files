use std::{path::PathBuf, process};

#[cfg(target_os = "linux")]
pub fn open_command(path: &PathBuf) -> process::Command {
    let mut command = process::Command::new("xdg-open");
    command.arg(path);
    command
}

#[cfg(target_os = "macos")]
pub fn open_command(path: &PathBuf) -> process::Command {
    let mut command = process::Command::new("open");
    command.arg(path);
    command
}

#[cfg(target_os = "redox")]
pub fn open_command(path: &PathBuf) -> process::Command {
    let mut command = process::Command::new("launcher");
    command.arg(path);
    command
}

#[cfg(target_os = "windows")]
pub fn open_command(path: &PathBuf) -> process::Command {
    use std::os::windows::process::CommandExt;

    let mut command = process::Command::new("cmd");

    command
        .arg("/c")
        .arg("start")
        .raw_arg("\"\"")
        .arg(path)
        .creation_flags(0x08000000);
    command
}
