use std::io;
use std::path::Path;

/// Attempt to perform a reflink (copy-on-write) copy of a file
pub async fn reflink_copy(from: &Path, to: &Path) -> Result<(), io::Error> {
    // Use blocking IO for the reflink operation since compio doesn't expose ioctl
    let from_path = from.to_path_buf();
    let to_path = to.to_path_buf();

    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "linux")]
        {
            use libc::{ioctl, FICLONE};
            use std::fs::File;
            use std::os::unix::io::AsRawFd;

            let src_file = File::open(&from_path)?;
            let dst_file = File::create(&to_path)?;

            let src_fd = src_file.as_raw_fd();
            let dst_fd = dst_file.as_raw_fd();

            // Call the FICLONE ioctl to perform the reflink copy
            let result = unsafe { ioctl(dst_fd, FICLONE, src_fd) };

            if result == 0 {
                Ok(())
            } else {
                // Clean up on failure - remove the empty file
                let _ = std::fs::remove_file(&to_path);
                let errno = unsafe { *libc::__errno_location() };
                Err(io::Error::from_raw_os_error(errno))
            }
        }

        // For non-Linux systems, return an error indicating reflink is not supported
        #[cfg(not(target_os = "linux"))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Reflink copy not supported on this platform",
            ))
        }
    })
    .await
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
}
