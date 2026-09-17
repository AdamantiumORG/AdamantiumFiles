//! Small, synchronous file operations. This Rust API is not an Adamantium ABI.
//!
//! Paths use host filesystem semantics, including symlink handling. On WASI,
//! access requires directories supplied by the host. Errors are preserved.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenMode {
    Read,
    Write,
    Append,
    ReadWrite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileMetadata {
    pub kind: EntryKind,
    pub length: u64,
    pub readonly: bool,
    pub modified_unix_seconds: Option<u64>,
}

/// Open a file using an explicit, cross-platform mode.
pub fn open(path: impl AsRef<Path>, mode: OpenMode) -> io::Result<File> {
    let mut options = OpenOptions::new();
    match mode {
        OpenMode::Read => {
            options.read(true);
        }
        OpenMode::Write => {
            options.write(true).create(true).truncate(true);
        }
        OpenMode::Append => {
            options.append(true).create(true);
        }
        OpenMode::ReadWrite => {
            options.read(true).write(true);
        }
    }
    options.open(path)
}

/// Create a missing file without truncating an existing file.
pub fn create_file(path: impl AsRef<Path>) -> io::Result<()> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map(drop)
}

/// Read a complete file as bytes, including non-UTF-8 content.
pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    fs::read(path)
}

/// Read a complete UTF-8 file. Invalid UTF-8 returns `InvalidData`.
pub fn read_text(path: impl AsRef<Path>) -> io::Result<String> {
    fs::read_to_string(path)
}

/// Create or truncate a file. Parent directories must already exist.
pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    fs::write(path, contents)
}

/// Append bytes, creating the file if missing. Does not add a newline.
pub fn append(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(contents.as_ref())
}

/// Check existence, following symlinks and preserving errors other than NotFound.
pub fn exists(path: impl AsRef<Path>) -> io::Result<bool> {
    path.as_ref().try_exists()
}

pub fn file_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub fn dir_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

/// Copy file contents and permissions, replacing the destination if present.
/// Returns bytes copied. Source and destination must refer to different files.
pub fn copy(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<u64> {
    fs::copy(source, destination)
}

/// Rename a file or directory using OS semantics. Cross-filesystem moves can fail.
/// Replacement behavior is platform-dependent; use an absent destination.
pub fn rename(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    fs::rename(source, destination)
}

/// Remove a file (or a symlink according to host semantics), never a directory tree.
pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    fs::remove_file(path)
}

/// Create a directory and missing parents. Existing directories are accepted.
pub fn create_dir(path: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(path)
}

/// Remove an empty directory. Nonempty directories are left intact.
pub fn remove_dir(path: impl AsRef<Path>) -> io::Result<()> {
    fs::remove_dir(path)
}

/// List immediate entry names, sorted using the platform's native path ordering.
/// Names are not converted to UTF-8 and entries are not traversed recursively.
pub fn list_dir(path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(path)?
        .map(|entry| entry.map(|entry| PathBuf::from(entry.file_name())))
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort();
    Ok(entries)
}

/// Read portable metadata without following platform-specific extension fields.
pub fn metadata(path: impl AsRef<Path>) -> io::Result<FileMetadata> {
    let metadata = fs::symlink_metadata(path)?;
    let file_type = metadata.file_type();
    let kind = if file_type.is_file() {
        EntryKind::File
    } else if file_type.is_dir() {
        EntryKind::Directory
    } else if file_type.is_symlink() {
        EntryKind::Symlink
    } else {
        EntryKind::Other
    };
    let modified_unix_seconds = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());
    Ok(FileMetadata {
        kind,
        length: metadata.len(),
        readonly: metadata.permissions().readonly(),
        modified_unix_seconds,
    })
}

/// Stable category used by the command ABI. The operating-system message is
/// still returned separately for diagnostics.
pub fn error_code(error: &io::Error) -> &'static str {
    match error.kind() {
        io::ErrorKind::NotFound => "not_found",
        io::ErrorKind::PermissionDenied => "permission_denied",
        io::ErrorKind::AlreadyExists => "already_exists",
        io::ErrorKind::InvalidInput | io::ErrorKind::InvalidData => "invalid_input",
        io::ErrorKind::NotADirectory => "not_a_directory",
        io::ErrorKind::IsADirectory => "is_a_directory",
        io::ErrorKind::DirectoryNotEmpty => "directory_not_empty",
        io::ErrorKind::ReadOnlyFilesystem => "read_only",
        _ => "io_error",
    }
}
