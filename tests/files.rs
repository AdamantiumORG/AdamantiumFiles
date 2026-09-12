use adamantium_files as files;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-data");
        std::fs::create_dir_all(&base).unwrap();
        loop {
            let path = base.join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("{error}"),
            }
        }
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn text_creation_overwrite_append_and_empty_contents() {
    let root = Workspace::new();
    let path = root.0.join("hello world-ą.txt");
    assert!(!files::exists(&path).unwrap());
    files::append(&path, "Hello ą").unwrap();
    files::append(&path, "\n世界").unwrap();
    assert_eq!(files::read_text(&path).unwrap(), "Hello ą\n世界");
    files::write(&path, "short").unwrap();
    assert_eq!(files::read(&path).unwrap(), b"short");
    files::write(&path, "").unwrap();
    assert!(files::read(&path).unwrap().is_empty());
    assert!(files::exists(&path).unwrap());
}

#[test]
fn binary_copy_rename_and_delete() {
    let root = Workspace::new();
    let source = root.0.join("source");
    let copy = root.0.join("copy");
    let moved = root.0.join("moved");
    files::write(&source, [0, 255, 128, 10]).unwrap();
    assert_eq!(
        files::read_text(&source).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    files::write(&copy, "old longer contents").unwrap();
    assert_eq!(files::copy(&source, &copy).unwrap(), 4);
    files::rename(&copy, &moved).unwrap();
    assert!(!files::exists(&copy).unwrap());
    assert_eq!(files::read(&moved).unwrap(), [0, 255, 128, 10]);
    files::remove_file(&moved).unwrap();
    assert!(!files::exists(&moved).unwrap());
    assert!(files::exists(&source).unwrap());
}

#[test]
fn directories_are_sorted_nonrecursive_and_only_removed_when_empty() {
    let root = Workspace::new();
    let nested = root.0.join("a/child");
    files::create_dir(&nested).unwrap();
    files::create_dir(&nested).unwrap();
    files::write(root.0.join("z"), "keep").unwrap();
    assert_eq!(
        files::list_dir(&root.0).unwrap(),
        vec![PathBuf::from("a"), PathBuf::from("z")]
    );
    assert!(files::remove_dir(&root.0).is_err());
    assert!(files::remove_file(root.0.join("a")).is_err());
    assert_eq!(files::read(root.0.join("z")).unwrap(), b"keep");
    files::remove_dir(&nested).unwrap();
    assert!(!files::exists(&nested).unwrap());
}

#[test]
fn errors_are_preserved_and_parents_are_not_implicitly_created_by_write() {
    let root = Workspace::new();
    let missing = root.0.join("missing");
    assert_eq!(
        files::read(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        files::remove_file(&missing).unwrap_err().kind(),
        ErrorKind::NotFound
    );
    assert!(files::list_dir(&missing).is_err());
    assert!(files::copy(&missing, root.0.join("copy")).is_err());
    assert!(files::rename(&missing, root.0.join("renamed")).is_err());
    assert!(files::write(missing.join("file"), "data").is_err());
    assert!(!files::exists(&missing).unwrap());
    files::write(root.0.join("file"), "data").unwrap();
    assert!(files::create_dir(root.0.join("file/child")).is_err());
}

#[test]
fn cli_validates_arguments_before_modifying_files_and_reports_io_errors() {
    let root = Workspace::new();
    let path = root.0.join("file");
    files::write(&path, "original").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_adamantium-files"))
        .arg("write")
        .arg(&path)
        .args(["replacement", "extra"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(files::read_text(&path).unwrap(), "original");
    let output = Command::new(env!("CARGO_BIN_EXE_adamantium-files"))
        .arg("read")
        .arg(root.0.join("missing"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("read:"));
}
