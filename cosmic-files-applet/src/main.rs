mod file_manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //TODO: move file manager service to its own daemon?
    let _conn_res = zbus::blocking::connection::Builder::session()?
        .name("org.freedesktop.FileManager1")?
        .serve_at("/org/freedesktop/FileManager1", file_manager::FileManager)?
        .build();

    cosmic_files::desktop()
}
