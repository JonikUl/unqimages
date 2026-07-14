use std::fs;
use std::io::Write;
use std::path::PathBuf;
use unqimages_core::{find_exact_duplicates, Config, DuplicateKind};

fn write_file(path: &PathBuf, content: &[u8]) {
    let mut file = fs::File::create(path).unwrap();
    file.write_all(content).unwrap();
}

#[test]
fn finds_exact_duplicates_and_ignores_uniques() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    write_file(&root.join("a.png"), b"duplicate content");
    write_file(&root.join("b.png"), b"duplicate content");
    write_file(&root.join("c.png"), b"unique content");

    let config = Config::new([root]);
    let groups = find_exact_duplicates(&config).unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].kind, DuplicateKind::Exact);
    assert_eq!(groups[0].entries.len(), 2);

    let paths: Vec<_> = groups[0]
        .entries
        .iter()
        .map(|e| e.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(paths.contains(&"a.png".to_string()));
    assert!(paths.contains(&"b.png".to_string()));
}

#[test]
fn exclude_dirs_skip_nested_folders() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let nested = root.join("nested");
    fs::create_dir(&nested).unwrap();

    write_file(&root.join("a.png"), b"duplicate content");
    write_file(&nested.join("b.png"), b"duplicate content");

    let config = Config {
        include_dirs: vec![root.to_path_buf()],
        exclude_dirs: vec![nested],
        extensions: vec![],
    };

    let groups = find_exact_duplicates(&config).unwrap();
    assert!(groups.is_empty());
}

#[test]
fn extension_filter_is_case_insensitive() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    write_file(&root.join("lower.png"), b"x");
    write_file(&root.join("upper.PNG"), b"x");
    write_file(&root.join("ignored.jpg"), b"x");

    let config = Config {
        include_dirs: vec![root.to_path_buf()],
        exclude_dirs: vec![],
        extensions: vec!["png".to_string()],
    };

    let groups = find_exact_duplicates(&config).unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].entries.len(), 2);
}
