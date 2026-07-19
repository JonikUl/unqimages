use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_unqimages-core")
}

fn write_file(path: &PathBuf, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn run(args: &[&str], cwd: &std::path::Path) -> (i32, String, String) {
    let output = Command::new(bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to spawn binary");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

#[test]
fn no_config_no_images_exit_zero() {
    let dir = tempfile::tempdir().unwrap();
    let (code, stdout, _) = run(&[], dir.path());
    assert_eq!(code, 0);
    assert!(stdout.contains("\"duplicates\": []"));
    assert!(stdout.contains("\"scanned\": 0"));
}

#[test]
fn json_output_finds_exact_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("public")).unwrap();
    write_file(&dir.path().join("public/a.png"), b"dup");
    write_file(&dir.path().join("public/b.png"), b"dup");

    let (code, stdout, _) = run(&[], dir.path());
    assert_eq!(code, 0);
    assert!(stdout.contains("\"duplicates\""));
    assert!(stdout.contains("public/a.png"));
    assert!(stdout.contains("public/b.png"));
    assert!(stdout.contains("\"scanned\": 2"));
}

#[test]
fn fail_on_duplicates_returns_exit_code_one() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("public")).unwrap();
    write_file(&dir.path().join("public/a.png"), b"dup");
    write_file(&dir.path().join("public/b.png"), b"dup");

    let config = dir.path().join("unqimages.json");
    fs::write(&config, r#"{"fail_on_duplicates": true}"#).unwrap();

    let (code, _, _) = run(&["--config", config.to_str().unwrap()], dir.path());
    assert_eq!(code, 1);
}

#[test]
fn table_output_contains_paths() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("public")).unwrap();
    write_file(&dir.path().join("public/a.png"), b"dup");
    write_file(&dir.path().join("public/b.png"), b"dup");

    let (code, stdout, _) = run(&["--output", "table"], dir.path());
    assert_eq!(code, 0);
    assert!(stdout.contains("Exact:"));
    assert!(stdout.contains("public/a.png"));
    assert!(stdout.contains("public/b.png"));
}

#[test]
fn missing_config_returns_exit_code_two() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.json");
    let (code, _, stderr) = run(&["--config", missing.to_str().unwrap()], dir.path());
    assert_eq!(code, 2);
    assert!(stderr.contains("config file does not exist"));
}

#[test]
fn missing_cwd_returns_exit_code_two() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing");
    let (code, _, stderr) = run(&["--cwd", missing.to_str().unwrap()], dir.path());
    assert_eq!(code, 2);
    assert!(stderr.contains("working directory does not exist"));
}

#[test]
fn invalid_output_format_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let (code, _, _) = run(&["--output", "yaml"], dir.path());
    assert_eq!(code, 2);
}

#[test]
fn no_cache_flag_ignores_cache_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("public")).unwrap();
    write_file(&dir.path().join("public/a.png"), b"dup");
    write_file(&dir.path().join("public/b.png"), b"dup");

    // First run builds the cache.
    let (code, stdout, _) = run(&[], dir.path());
    assert_eq!(code, 0);
    assert!(stdout.contains("\"used_cache\": false"));

    // Second run with --no-cache should not use the cache.
    let (code, stdout, _) = run(&["--no-cache", "--output", "json"], dir.path());
    assert_eq!(code, 0);
    assert!(stdout.contains("\"used_cache\": false"));
}

fn init_git_repo(dir: &std::path::Path) {
    let status = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .status()
        .expect("failed to run git init");
    assert!(status.success());

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .status()
        .unwrap();
}

fn git_commit(dir: &std::path::Path, message: &str) {
    let status = Command::new("git")
        .args(["commit", "-q", "-m", message])
        .current_dir(dir)
        .status()
        .expect("failed to run git commit");
    assert!(status.success());
}

fn git_add(dir: &std::path::Path, path: &str) {
    let status = Command::new("git")
        .args(["add", path])
        .current_dir(dir)
        .status()
        .expect("failed to run git add");
    assert!(status.success());
}

#[test]
fn staged_no_files_exit_zero() {
    let dir = tempfile::tempdir().unwrap();
    init_git_repo(dir.path());

    let (code, _, stderr) = run(&["--staged"], dir.path());
    assert_eq!(code, 0);
    assert!(stderr.contains("no staged image files"));
}

#[test]
fn staged_finds_duplicate_of_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("public")).unwrap();
    write_file(&dir.path().join("public/existing.png"), b"dup");
    init_git_repo(dir.path());
    git_add(dir.path(), "public/existing.png");
    git_commit(dir.path(), "initial");

    write_file(&dir.path().join("public/staged.png"), b"dup");
    git_add(dir.path(), "public/staged.png");

    let config = dir.path().join("unqimages.json");
    fs::write(&config, r#"{"fail_on_duplicates": true}"#).unwrap();

    let (code, stdout, _) = run(
        &["--staged", "--config", config.to_str().unwrap()],
        dir.path(),
    );
    assert_eq!(code, 1);
    assert!(stdout.contains("public/existing.png"));
    assert!(stdout.contains("public/staged.png"));
    assert!(stdout.contains("\"scanned\": 1"));
}

#[test]
fn staged_with_explicit_paths_finds_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("public")).unwrap();
    write_file(&dir.path().join("public/a.png"), b"dup");
    write_file(&dir.path().join("public/b.png"), b"dup");

    let (code, stdout, _) = run(&["--staged", "public/a.png", "public/b.png"], dir.path());
    assert_eq!(code, 0);
    assert!(stdout.contains("public/a.png"));
    assert!(stdout.contains("public/b.png"));
    assert!(stdout.contains("\"scanned\": 2"));
}

#[test]
fn staged_does_not_report_existing_duplicates_without_staged_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("public")).unwrap();
    write_file(&dir.path().join("public/a.png"), b"dup");
    write_file(&dir.path().join("public/b.png"), b"dup");
    init_git_repo(dir.path());
    git_add(dir.path(), ".");
    git_commit(dir.path(), "initial");

    // Stage a new unique file; the existing duplicate pair should not be reported
    // because it does not involve the staged file.
    write_file(&dir.path().join("public/staged.png"), b"unique");
    git_add(dir.path(), "public/staged.png");

    let config = dir.path().join("unqimages.json");
    fs::write(&config, r#"{"fail_on_duplicates": true}"#).unwrap();

    let (code, stdout, _) = run(
        &["--staged", "--config", config.to_str().unwrap()],
        dir.path(),
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("\"duplicates\": []"));
}

#[test]
fn staged_does_not_double_count_when_include_dirs_are_absolute() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("public")).unwrap();
    write_file(&dir.path().join("public/existing.png"), b"dup");
    init_git_repo(dir.path());
    git_add(dir.path(), ".");
    git_commit(dir.path(), "initial");

    write_file(&dir.path().join("public/staged.png"), b"dup");
    git_add(dir.path(), "public/staged.png");

    let config = dir.path().join("unqimages.json");
    let include_dir = dir.path().join("public").to_string_lossy().to_string();
    fs::write(
        &config,
        format!(
            r#"{{"include_dirs": ["{}"], "fail_on_duplicates": true}}"#,
            include_dir
        ),
    )
    .unwrap();

    let (code, stdout, _) = run(
        &["--staged", "--config", config.to_str().unwrap()],
        dir.path(),
    );
    assert_eq!(code, 1);

    // The staged file must appear exactly once, not as both an absolute and a
    // relative path.
    let staged_count = stdout.matches("staged.png").count();
    assert_eq!(staged_count, 1);
}
