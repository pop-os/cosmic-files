use crate::{
    mime_icon::mime_for_path,
    operation::{Controller, OpReader, OperationError, OperationErrorType, sync_to_disk},
};
use cosmic::iced::futures;
use jiff::{Zoned, civil::DateTime, tz::TimeZone};
use rustc_hash::FxHashSet;
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};
use zip::result::ZipError;

pub const SUPPORTED_ARCHIVE_TYPES: &[&str] = &[
    "application/gzip",
    "application/x-compressed-tar",
    "application/x-tar",
    "application/zip",
    #[cfg(feature = "bzip2")]
    "application/x-bzip",
    #[cfg(feature = "bzip2")]
    "application/x-bzip-compressed-tar",
    #[cfg(feature = "bzip2")]
    "application/x-bzip2",
    #[cfg(feature = "bzip2")]
    "application/x-bzip2-compressed-tar",
    #[cfg(feature = "lzma-rust2")]
    "application/x-xz",
    #[cfg(feature = "lzma-rust2")]
    "application/x-xz-compressed-tar",
];

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    ".tar.bz2",
    ".tar.gz",
    ".tar.lzma",
    ".tar.xz",
    ".tgz",
    ".tar",
    ".zip",
];

pub fn extract(
    path: &Path,
    new_dir: &Path,
    password: Option<&str>,
    controller: &Controller,
) -> Result<(), OperationError> {
    let mime = mime_for_path(path, None, false);
    match mime.essence_str() {
        "application/gzip" | "application/x-compressed-tar" => {
            OpReader::new(path, controller.clone())
                .map(io::BufReader::new)
                .map(flate2::read::GzDecoder::new)
                .map(tar::Archive::new)
                .and_then(|mut archive| archive.unpack(new_dir))
                .map_err(|e| OperationError::from_err(e, controller))
        }
        "application/x-tar" => OpReader::new(path, controller.clone())
            .map(io::BufReader::new)
            .map(tar::Archive::new)
            .and_then(|mut archive| archive.unpack(new_dir))
            .map_err(|e| OperationError::from_err(e, controller)),
        "application/zip" => fs::File::open(path)
            .map(io::BufReader::new)
            .map(zip::ZipArchive::new)
            .map_err(|e| OperationError::from_err(e, controller))?
            .and_then(move |mut archive| zip_extract(&mut archive, new_dir, password, controller))
            .map_err(|e| match e {
                ZipError::UnsupportedArchive(ZipError::PASSWORD_REQUIRED)
                | ZipError::InvalidPassword => {
                    OperationError::from_kind(OperationErrorType::PasswordRequired, controller)
                }
                _ => OperationError::from_err(e, controller),
            }),
        #[cfg(feature = "bzip2")]
        "application/x-bzip"
        | "application/x-bzip-compressed-tar"
        | "application/x-bzip2"
        | "application/x-bzip2-compressed-tar" => OpReader::new(path, controller.clone())
            .map(io::BufReader::new)
            .map(bzip2::read::BzDecoder::new)
            .map(tar::Archive::new)
            .and_then(|mut archive| archive.unpack(new_dir))
            .map_err(|e| OperationError::from_err(e, controller)),
        #[cfg(feature = "lzma-rust2")]
        "application/x-xz" | "application/x-xz-compressed-tar" => {
            OpReader::new(path, controller.clone())
                .map(io::BufReader::new)
                .map(|reader| lzma_rust2::XzReader::new(reader, true))
                .map(tar::Archive::new)
                .and_then(|mut archive| archive.unpack(new_dir))
                .map_err(|e| OperationError::from_err(e, controller))
        }
        _ => Err(OperationError::from_err(
            format!("unsupported mime type {mime:?}"),
            controller,
        )),
    }
}

// From https://docs.rs/zip/latest/zip/read/struct.ZipArchive.html#method.extract, with cancellation and progress added
fn zip_extract<R: io::Read + io::Seek, P: AsRef<Path>>(
    archive: &mut zip::ZipArchive<R>,
    directory: P,
    password: Option<&str>,
    controller: &Controller,
) -> zip::result::ZipResult<()> {
    use std::{ffi::OsString, fs};
    use zip::result::ZipError;

    fn make_writable_dir_all<T: AsRef<Path>>(
        outpath: T,
        target_dirs: &mut FxHashSet<PathBuf>,
    ) -> Result<(), ZipError> {
        let path = outpath.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        target_dirs.insert(path.to_path_buf());

        #[cfg(unix)]
        {
            // Dirs must be writable until all normal files are extracted
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                path,
                fs::Permissions::from_mode(0o700 | fs::metadata(path)?.permissions().mode()),
            )?;
        }
        Ok(())
    }

    let mut buffer = vec![0; 4 * 1024 * 1024];
    let total_files = archive.len();
    let mut written_files = Vec::with_capacity(total_files);
    let mut target_dirs = FxHashSet::default();
    #[cfg(unix)]
    let mut files_by_unix_mode = Vec::with_capacity(total_files);
    let mut files_by_last_modified = Vec::with_capacity(total_files);

    for i in 0..total_files {
        futures::executor::block_on(controller.check())
            .map_err(|s| io::Error::other(OperationError::from_state(s, controller)))?;

        controller.set_progress(i as f32 / total_files as f32);

        let mut file = match password {
            None => archive.by_index(i),
            Some(pwd) => archive.by_index_decrypt(i, pwd.as_bytes()),
        }?;

        let filepath = file
            .enclosed_name()
            .ok_or(ZipError::InvalidArchive("Invalid file path".into()))?;

        let outpath = directory.as_ref().join(filepath);

        if let Some(last_modified) = file.last_modified() {
            files_by_last_modified.push((outpath.clone(), last_modified));
        }

        if file.is_dir() {
            make_writable_dir_all(&outpath, &mut target_dirs)?;

            #[cfg(unix)]
            if let Some(mode) = file.unix_mode() {
                files_by_unix_mode.push((outpath, mode));
            }
            continue;
        }

        if let Some(parent) = outpath.parent() {
            make_writable_dir_all(parent, &mut target_dirs)?;
        }

        if file.is_symlink() && (cfg!(unix) || cfg!(windows)) {
            let mut target = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut target)?;
            // File no longer needed, drop to allow reading target on windows
            drop(file);

            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStringExt;
                let target = OsString::from_vec(target);
                std::os::unix::fs::symlink(&target, outpath.as_path())?;
            }
            #[cfg(windows)]
            {
                let Ok(target) = String::from_utf8(target) else {
                    return Err(ZipError::InvalidArchive(
                        "Invalid UTF-8 as symlink target".into(),
                    ));
                };
                let target_is_dir_from_archive = match password {
                    None => archive.by_name(&target),
                    Some(pwd) => archive.by_name_decrypt(&target, pwd.as_bytes()),
                }
                .map_or(false, |x| x.is_dir());
                let target_path = directory.as_ref().join(OsString::from(target.to_string()));
                let target_is_dir = if target_is_dir_from_archive {
                    true
                } else if let Ok(meta) = std::fs::metadata(&target_path) {
                    meta.is_dir()
                } else {
                    false
                };
                if target_is_dir {
                    std::os::windows::fs::symlink_dir(target_path, outpath.as_path())?;
                } else {
                    std::os::windows::fs::symlink_file(target_path, outpath.as_path())?;
                }
            }

            written_files.push(outpath);
            continue;
        }

        let total = file.size();
        let mut outfile = fs::File::create(&outpath)?;
        let mut current = 0;
        loop {
            futures::executor::block_on(controller.check())
                .map_err(|s| io::Error::other(OperationError::from_state(s, controller)))?;

            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            outfile.write_all(&buffer[..count])?;
            current += count as u64;

            if current < total {
                let file_progress = current as f32 / total as f32;
                let total_progress = (i as f32 + file_progress) / total_files as f32;
                controller.set_progress(total_progress);
            }
        }

        // Check for real permissions, which we'll set in a second pass
        #[cfg(unix)]
        if let Some(mode) = file.unix_mode() {
            files_by_unix_mode.push((outpath.clone(), mode));
        }

        written_files.push(outpath);
    }
    #[cfg(unix)]
    {
        use std::cmp::Reverse;
        use std::os::unix::fs::PermissionsExt;

        if files_by_unix_mode.len() > 1 {
            // Ensure we update children's permissions before making a parent unwritable
            files_by_unix_mode.sort_by_key(|(path, _)| Reverse(path.components().count()));
        }
        for (path, mode) in files_by_unix_mode {
            fs::set_permissions(&path, fs::Permissions::from_mode(mode))?;
        }
    }

    for (path, last_modified) in files_by_last_modified {
        if let Some(modified) = zip_date_time_to_system_time(last_modified) {
            let file_time = filetime::FileTime::from_system_time(modified);
            filetime::set_file_mtime(&path, file_time)?;
        }
    }

    // Flush files to disk
    futures::executor::block_on(sync_to_disk(written_files, target_dirs));

    Ok(())
}

fn zip_date_time_to_system_time(date_time: zip::DateTime) -> Option<SystemTime> {
    let dt = DateTime::new(
        date_time.year() as i16,
        date_time.month() as i8,
        date_time.day() as i8,
        date_time.hour() as i8,
        date_time.minute() as i8,
        date_time.second() as i8,
        0,
    )
    .ok()?;
    TimeZone::system()
        .to_ambiguous_zoned(dt)
        .later()
        .ok()
        .map(SystemTime::from)
}

pub fn system_time_to_zip_date_time(system_time: SystemTime) -> Option<zip::DateTime> {
    let date_time = Zoned::try_from(system_time).ok()?;

    zip::DateTime::from_date_and_time(
        date_time.year().unsigned_abs(),
        date_time.month().unsigned_abs(),
        date_time.day().unsigned_abs(),
        date_time.hour().unsigned_abs(),
        date_time.minute().unsigned_abs(),
        date_time.second().unsigned_abs(),
    )
    .ok()
}
