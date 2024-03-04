use std::{io, process};

// This code is from the open crate and retains its MIT license.
pub fn spawn_detached(command: &mut process::Command) -> io::Result<()> {
    command
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null());

    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt as _;

        command.pre_exec(move || {
            match libc::fork() {
                -1 => return Err(io::Error::last_os_error()),
                0 => (),
                _ => libc::_exit(0),
            }

            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }

            Ok(())
        });
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
    }

    command.spawn().map(|_| ())
}
