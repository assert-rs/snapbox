use super::*;

#[test]
fn strips_trailing_slash() {
    let path = std::path::Path::new("/foo/bar/");
    let rendered = path.display().to_string();
    assert_eq!(rendered.as_bytes()[rendered.len() - 1], b'/');

    let stripped = strip_trailing_slash(path);
    let rendered = stripped.display().to_string();
    assert_eq!(rendered.as_bytes()[rendered.len() - 1], b'r');
}

#[test]
fn file_type_detect_file() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    dbg!(&path);
    let actual = FileType::from_path(&path);
    assert_eq!(actual, FileType::File);
}

#[test]
fn file_type_detect_dir() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    dbg!(path);
    let actual = FileType::from_path(path);
    assert_eq!(actual, FileType::Dir);
}

#[test]
fn file_type_detect_missing() {
    let path = std::path::Path::new("this-should-never-exist");
    dbg!(path);
    let actual = FileType::from_path(path);
    assert_eq!(actual, FileType::Missing);
}
