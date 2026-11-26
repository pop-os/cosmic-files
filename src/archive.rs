use crate::mime_icon::mime_for_path;
use crate::operation::{
    Controller, OpReader, OperationError, OperationErrorType, SYNC_BATCH, copy_unique_path,
};
use cosmic::iced::futures;
use jiff::Zoned;
use jiff::civil::DateTime;
use jiff::tz::TimeZone;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
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
    #[cfg(feature = "lzma-rust2")]
    "application/x-lzma",
    #[cfg(feature = "lzma-rust2")]
    "application/x-lzma-compressed-tar",
];

// Order matters. Compound extensions must come before their single extensions.
const SUPPORTED_EXTENSIONS: &[&str] = &[
    ".tar.bz2",
    ".tar.gz",
    ".tar.lzma",
    ".tar.xz",
    ".tgz",
    ".tar",
    ".zip",
    ".bz2",
    ".gz",
    ".lzma",
    ".xz",
];

struct StoredMetadata {
    file_name: Option<Vec<u8>>,
    mtime: Option<SystemTime>,
}

pub fn extract(
    path: &Path,
    to: &Path,
    password: &Option<String>,
    controller: &Controller,
) -> Result<PathBuf, OperationError> {
    let op_reader = || OpReader::new(path, controller.clone()).map(io::BufReader::new);
    let operation_error = |e| OperationError::from_err(e, controller);
    let mime = mime_for_path(path, None, false);
    let password = password.as_deref();
    match mime.essence_str() {
        "application/gzip" | "application/x-compressed-tar" => {
            op_reader()
                .map(flate2::read::MultiGzDecoder::new)
                .and_then(|decoder| {
                    extract_compressed(
                        path,
                        to,
                        decoder,
                        // gzip sometimes has filename and mtime that we can use
                        |decoder| {
                            Some(StoredMetadata {
                                file_name: decoder
                                    .header()
                                    .and_then(|h| h.filename())
                                    .map(<[u8]>::to_vec),
                                mtime: decoder.header().map(|h| h.mtime()).filter(|&t| t != 0).map(
                                    |t| SystemTime::UNIX_EPOCH + Duration::from_secs(t.into()),
                                ),
                            })
                        },
                    )
                })
                .map_err(operation_error)
        }
        "application/x-tar" => {
            let dir = unique_extraction_dir(path, to);
            op_reader()
                .map(tar::Archive::new)
                .and_then(|mut archive| archive.unpack(&dir))
                .map(|_| dir)
                .map_err(operation_error)
        }
        "application/zip" => {
            let dir = unique_extraction_dir(path, to);
            fs::File::open(path)
                .map(io::BufReader::new)
                .map(zip::ZipArchive::new)
                .map_err(|e| OperationError::from_err(e, controller))?
                .and_then(|mut archive| {
                    zip_extract(&mut archive, &dir, password, controller.clone())
                })
                .map(|_| dir)
                .map_err(|e| match e {
                    ZipError::UnsupportedArchive(ZipError::PASSWORD_REQUIRED)
                    | ZipError::InvalidPassword => {
                        OperationError::from_kind(OperationErrorType::PasswordRequired, controller)
                    }
                    _ => OperationError::from_err(e, controller),
                })
        }
        #[cfg(feature = "bzip2")]
        "application/x-bzip"
        | "application/x-bzip-compressed-tar"
        | "application/x-bzip2"
        | "application/x-bzip2-compressed-tar" => op_reader()
            .map(bzip2::read::MultiBzDecoder::new)
            .and_then(|decoder| extract_compressed(path, to, decoder, |_| None))
            .map_err(operation_error),
        #[cfg(feature = "lzma-rust2")]
        "application/x-xz" | "application/x-xz-compressed-tar" => op_reader()
            .map(|reader| lzma_rust2::XzReader::new(reader, true))
            .and_then(|decoder| extract_compressed(path, to, decoder, |_| None))
            .map_err(operation_error),
        #[cfg(feature = "lzma-rust2")]
        "application/x-lzma" | "application/x-lzma-compressed-tar" => op_reader()
            .map(|reader| lzma_rust2::LzmaReader::new_mem_limit(reader, u32::MAX, None))
            .and_then(|decoder| extract_compressed(path, to, decoder?, |_| None))
            .map_err(operation_error),
        _ => Err(OperationError::from_err(
            format!("unsupported mime type {mime:?}"),
            controller,
        )),
    }
}

/// Extracts from a compressed file that could be either a single file or a tar archive.
/// For a single file, the file gets extracted with a name and mtime from the metadata
/// if it exists, but falls back to deriving from the original file.
/// If it's a tar, all of the files get extracted to a directory named from the original file.
fn extract_compressed<D: Read>(
    path: &Path,
    to: &Path,
    mut decoder: D,
    stored_metadata: impl FnOnce(&D) -> Option<StoredMetadata>,
) -> io::Result<PathBuf> {
    let metadata = stored_metadata(&decoder);
    let mut header = Vec::with_capacity(512);
    decoder.by_ref().take(512).read_to_end(&mut header)?;

    // This simple check if it's a tar will give a false positive for an
    // unlikely single file that just happens to have "ustar" at this position.
    // For a more robust check we could compute and compare the checksum of the header, but this
    // simpler check is probably good enough.
    let is_tar = header.len() >= 262 && &header[257..262] == b"ustar";
    let decoder = Cursor::new(header).chain(decoder);

    if is_tar {
        let dir = unique_extraction_dir(path, to);
        tar::Archive::new(decoder).unpack(&dir)?;
        Ok(dir)
    } else {
        let (stored_file_name, stored_mtime) =
            metadata.map(|m| (m.file_name, m.mtime)).unwrap_or_default();
        let out_path = unique_single_file_path(path, to, stored_file_name);
        extract_single_file(path, out_path, decoder, stored_mtime)
    }
}

fn extract_single_file<D: Read>(
    path: &Path,
    out_path: PathBuf,
    mut decoder: D,
    stored_mtime: Option<SystemTime>,
) -> io::Result<PathBuf> {
    let mut file = fs::File::create(&out_path)?;
    if let Err(e) = io::copy(&mut decoder, &mut file) {
        drop(file);
        let _ = fs::remove_file(&out_path);
        return Err(e);
    }

    match io::copy(&mut decoder, &mut file) {
        Ok(_) => {
            if let Some(mtime) = stored_mtime {
                let _ = file.set_modified(mtime);
            } else if let Ok(metadata) = fs::metadata(path) {
                if let Ok(mtime) = metadata.modified() {
                    let _ = file.set_modified(mtime);
                }
            }
        }
        Err(e) => {
            drop(file);
            let _ = fs::remove_file(&out_path);
            return Err(e);
        }
    }

    Ok(out_path)
}

fn unique_extraction_dir(path: &Path, to: &Path) -> PathBuf {
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("archive");
    let file_stem = SUPPORTED_EXTENSIONS
        .iter()
        .find_map(|ext| filename.strip_suffix(ext))
        .unwrap_or(filename);

    copy_unique_path(&to.join(file_stem), to)
}

fn unique_single_file_path(path: &Path, to: &Path, stored_file_name: Option<Vec<u8>>) -> PathBuf {
    let file_name = single_file_name(path, stored_file_name);
    copy_unique_path(&to.join(file_name), to)
}

fn single_file_name(path: &Path, stored_file_name: Option<Vec<u8>>) -> String {
    // First try to get stored file name, but guard against malicious values.
    if let Some(ref raw) = stored_file_name {
        let s = String::from_utf8_lossy(&raw);
        if let Ok(p) = PathBuf::from_str(&s)
            && let Some(file_name) = p.file_name()
        {
            let file_name = file_name.to_string_lossy();
            if !file_name.is_empty() && file_name != "." && file_name != ".." {
                return file_name.to_string();
            }
        }
    }

    // Second try deriving the file name from the original name with the extension removed.
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("archive");
    let lower = file_name.to_ascii_lowercase();
    for ext in SUPPORTED_EXTENSIONS {
        if lower.ends_with(ext) && file_name.len() > ext.len() {
            return (&file_name[..file_name.len() - ext.len()]).to_string();
        }
    }

    // If all else fails, just append ".out" to the original file name.
    format!("{file_name}.out")
}

// From https://docs.rs/zip/latest/zip/read/struct.ZipArchive.html#method.extract, with cancellation and progress added
fn zip_extract<R: io::Read + io::Seek, P: AsRef<Path>>(
    archive: &mut zip::ZipArchive<R>,
    directory: P,
    password: Option<&str>,
    controller: Controller,
) -> zip::result::ZipResult<()> {
    use std::ffi::OsString;
    use std::fs;
    use zip::result::ZipError;

    fn make_writable_dir_all<T: AsRef<Path>>(
        outpath: T,
        target_dirs: &mut HashSet<PathBuf>,
    ) -> Result<(), ZipError> {
        let path = outpath.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        if !target_dirs.contains(path) {
            target_dirs.insert(path.to_path_buf());
        }

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
    let mut dirty_fds = Vec::with_capacity(SYNC_BATCH);
    let mut target_dirs = HashSet::new();
    #[cfg(unix)]
    let mut files_by_unix_mode = Vec::with_capacity(total_files);
    let mut files_by_last_modified = Vec::with_capacity(total_files);

    for i in 0..total_files {
        futures::executor::block_on(async {
            controller
                .check()
                .await
                .map_err(|s| io::Error::other(OperationError::from_state(s, &controller)))
        })?;

        controller.set_progress(i as f32 / total_files as f32);

        let mut file = match password {
            None => archive.by_index(i),
            Some(pwd) => archive.by_index_decrypt(i, pwd.as_bytes()),
        }?;

        let filepath = file
            .enclosed_name()
            .ok_or(ZipError::InvalidArchive("Invalid file path".into()))?;

        let outpath = directory.as_ref().join(filepath);

        if (file.is_dir() || file.is_symlink())
            && let Some(last_modified) = file.last_modified()
        {
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

            continue;
        }

        let total = file.size();
        let mut outfile = fs::File::create(&outpath)?;
        let mut current = 0;
        loop {
            futures::executor::block_on(async {
                controller
                    .check()
                    .await
                    .map_err(|s| io::Error::other(OperationError::from_state(s, &controller)))
            })?;

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

        #[cfg(unix)]
        if let Some(mode) = file.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            outfile.set_permissions(fs::Permissions::from_mode(mode))?;
        }
        if let Some(modified) = file.last_modified().and_then(zip_date_time_to_system_time) {
            outfile.set_modified(modified)?;
        }

        dirty_fds.push(outfile);
        if dirty_fds.len() >= SYNC_BATCH {
            for fd in dirty_fds.drain(..) {
                fd.sync_all()?;
            }
        }
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

    for fd in dirty_fds {
        fd.sync_all()?;
    }
    for dir in target_dirs {
        fs::File::open(dir)?.sync_all()?;
    }

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
        date_time.year() as u16,
        date_time.month() as u8,
        date_time.day() as u8,
        date_time.hour() as u8,
        date_time.minute() as u8,
        date_time.second() as u8,
    )
    .ok()
}
