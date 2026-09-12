//! Small, synchronous file operations. This Rust API is not an Adamantium ABI.
//!
//! Paths use host filesystem semantics, including symlink handling. On WASI,
//! access requires directories supplied by the host. Errors are preserved.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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
