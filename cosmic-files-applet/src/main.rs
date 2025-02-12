mod file_manager;

/// Access glibc malloc tunables.
#[cfg(target_env = "gnu")]
mod malloc {
    use std::os::raw::c_int;

    const M_MMAP_THRESHOLD: c_int = -3;

    extern "C" {
        fn mallopt(param: c_int, value: c_int) -> c_int;
    }

    /// Prevents glibc from hoarding memory via memory fragmentation.
    pub fn limit_mmap_threshold() {
        unsafe {
            mallopt(M_MMAP_THRESHOLD, 65536);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_env = "gnu")]
    malloc::limit_mmap_threshold();

    //TODO: move file manager service to its own daemon?
    let _conn_res = zbus::blocking::connection::Builder::session()?
        .name("org.freedesktop.FileManager1")?
        .serve_at("/org/freedesktop/FileManager1", file_manager::FileManager)?
        .build();

    cosmic_files::desktop()
}
