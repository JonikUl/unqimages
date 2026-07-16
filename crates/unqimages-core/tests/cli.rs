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
