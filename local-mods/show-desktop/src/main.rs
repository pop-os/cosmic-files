mod wm;
use std::{io::{Read, Write}, os::unix::net::{UnixListener, UnixStream}, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(std::env::var("XDG_RUNTIME_DIR")?).join("cosmic-show-desktop.sock");
    let status = std::env::args().any(|a| a == "--status");
    if status || std::env::args().any(|a| a == "--toggle") {
        let mut stream = UnixStream::connect(path)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(4)))?;
        stream.write_all(if status { b"status\n" } else { b"toggle\n" })?;
        let mut reply = String::new();
        stream.read_to_string(&mut reply)?;
        if status { println!("{reply}"); return Ok(()); }
        if reply.trim() != "ok" { return Err(reply.into()); }
        return Ok(());
    }
    if path.exists() {
        if UnixStream::connect(&path).is_ok() { return Ok(()); }
        std::fs::remove_file(&path)?;
    }
    let listener = UnixListener::bind(&path)?;
    let manager = wm::WindowManager::new()?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
        let mut command = [0; 32];
        let n = stream.read(&mut command)?;
        if &command[..n] == b"status\n" {
            let _ = stream.write_all(manager.status()?.as_bytes());
            continue;
        }
        if &command[..n] != b"toggle\n" { continue; }
        let reply = match manager.toggle() { Ok(()) => "ok".to_string(), Err(e) => format!("{e}") };
        let _ = stream.write_all(reply.as_bytes());
    }
    Ok(())
}
