use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use unqimages_core::{find_exact_duplicates, Config};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_unqimages-core")
}

fn write_file(path: &PathBuf, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn config_with_cache(root: &Path, cache_root: &Path) -> Config {
    Config {
        include_dirs: vec![root.to_path_buf()],
        cache_dir: Some(cache_root.to_path_buf()),
        extensions: vec![],
        ..Default::default()
    }
}

#[test]
fn cache_miss_reports_used_cache_false() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("images");
    fs::create_dir(&root).unwrap();
    write_file(&root.join("a.bin"), b"dup");
    write_file(&root.join("b.bin"), b"dup");

    let config = config_with_cache(&root, dir.path());
    let result = find_exact_duplicates(&config).unwrap();

    assert!(!result.used_cache);
    assert_eq!(result.scanned, 2);
}

#[test]
fn cache_hit_reports_used_cache_true() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("images");
    fs::create_dir(&root).unwrap();
    write_file(&root.join("a.bin"), b"dup");
    write_file(&root.join("b.bin"), b"dup");

    let config = config_with_cache(&root, dir.path());
    let first = find_exact_duplicates(&config).unwrap();
    assert!(!first.used_cache);

    let second = find_exact_duplicates(&config).unwrap();
    assert!(second.used_cache);
    assert_eq!(second.scanned, 2);
}

#[test]
fn cache_invalidates_when_file_changes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("images");
    fs::create_dir(&root).unwrap();
    let a = root.join("a.bin");
    write_file(&a, b"dup");
    write_file(&root.join("b.bin"), b"dup");

    let config = config_with_cache(&root, dir.path());
    let first = find_exact_duplicates(&config).unwrap();
    assert!(!first.used_cache);

    std::thread::sleep(std::time::Duration::from_millis(10));
    write_file(&a, b"changed");

    let second = find_exact_duplicates(&config).unwrap();
    // The unchanged file is still served from cache, so used_cache may be true.
    // The important part is that the changed file no longer forms a duplicate group.
    assert!(second.groups.is_empty());
}

#[test]
fn corrupted_cache_is_rebuilt() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("images");
    fs::create_dir(&root).unwrap();
    write_file(&root.join("a.bin"), b"dup");
    write_file(&root.join("b.bin"), b"dup");

    let config = config_with_cache(&root, dir.path());
    let _ = find_exact_duplicates(&config).unwrap();

    let cache_path = dir.path().join("cache.json");
    {
        let mut file = fs::File::create(&cache_path).unwrap();
        file.write_all(b"not json").unwrap();
    }

    let result = find_exact_duplicates(&config).unwrap();
    assert!(!result.used_cache);
    assert_eq!(result.scanned, 2);
    assert!(cache_path.exists());
}

#[test]
fn cli_no_cache_ignores_existing_cache() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join("public")).unwrap();
    write_file(&root.join("public/a.png"), b"dup");
    write_file(&root.join("public/b.png"), b"dup");

    let output = Command::new(bin())
        .current_dir(root)
        .output()
        .expect("failed to spawn binary");
    assert!(output.status.success());
    let first_stdout = String::from_utf8_lossy(&output.stdout);
    assert!(first_stdout.contains("\"used_cache\": false"));

    let output = Command::new(bin())
        .args(["--no-cache", "--output", "json"])
        .current_dir(root)
        .output()
        .expect("failed to spawn binary");
    assert!(output.status.success());
    let second_stdout = String::from_utf8_lossy(&output.stdout);
    assert!(second_stdout.contains("\"used_cache\": false"));
}

#[test]
fn concurrent_runs_leave_valid_cache() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("images");
    fs::create_dir(&root).unwrap();
    for i in 0..50 {
        write_file(&root.join(format!("file_{}.bin", i)), b"same");
    }

    let config = config_with_cache(&root, dir.path());

    let config_ref = &config;
    std::thread::scope(|s| {
        s.spawn(|| find_exact_duplicates(config_ref).unwrap());
        s.spawn(|| find_exact_duplicates(config_ref).unwrap());
    });

    let result = find_exact_duplicates(&config).unwrap();
    assert!(result.used_cache);
    assert_eq!(result.scanned, 50);
}
