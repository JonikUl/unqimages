use crate::{Config, ImageEntry};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Returns entries ordered by filesystem walk. Each entry leaves `file_hash`
/// and `perceptual_hash` unset; callers must compute them separately.
pub fn discover_images(config: &Config) -> Vec<ImageEntry> {
    let mut entries = Vec::new();

    for root in &config.include_dirs {
        // Symlinks are not followed to avoid cycles and double-counting the
        // same file through different paths.
        for result in WalkDir::new(root).follow_links(false).into_iter() {
            let Ok(entry) = result else { continue };
            let path = entry.path();

            if !entry.file_type().is_file() {
                continue;
            }

            if is_excluded(path, &config.exclude_dirs) {
                continue;
            }

            if !has_allowed_extension(path, &config.extensions) {
                continue;
            }

            let Some((size, modified)) = file_metadata(path) else {
                continue;
            };

            entries.push(ImageEntry {
                path: path.to_path_buf(),
                size,
                modified,
                file_hash: None,
                perceptual_hash: None,
            });
        }
    }

    entries
}

/// Apply project filters to caller-supplied staged paths.
///
/// Staged files must respect the same extension and exclusion rules as files
/// discovered by the normal scan.
pub fn discover_staged_images(config: &Config, staged_paths: &[PathBuf]) -> Vec<ImageEntry> {
    let mut entries = Vec::new();

    for path in staged_paths {
        let path = normalize_staged_path(path);

        if !path.is_file() {
            continue;
        }

        if is_excluded(&path, &config.exclude_dirs) {
            continue;
        }

        if !has_allowed_extension(&path, &config.extensions) {
            continue;
        }

        let Some((size, modified)) = file_metadata(&path) else {
            continue;
        };

        entries.push(ImageEntry {
            path,
            size,
            modified,
            file_hash: None,
            perceptual_hash: None,
        });
    }

    entries
}

fn normalize_staged_path(path: &Path) -> PathBuf {
    // Keep paths relative to the current directory when possible, matching the
    // output produced by `git diff --cached --name-only`.
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(stripped) = path.strip_prefix(&cwd) {
            return stripped.to_path_buf();
        }
    }
    path.to_path_buf()
}

fn file_metadata(path: &Path) -> Option<(u64, u64)> {
    let metadata = fs::metadata(path).ok()?;
    let size = metadata.len();
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as u64)
        // A pre-1970 mtime is treated as epoch to keep the cache schema
        // simple and monotonic.
        .unwrap_or(0);
    Some((size, modified))
}

fn is_excluded(path: &Path, exclude_dirs: &[PathBuf]) -> bool {
    exclude_dirs.iter().any(|exclude| path.starts_with(exclude))
}

fn has_allowed_extension(path: &Path, extensions: &[String]) -> bool {
    // An empty extension list means "accept all" so the default config works
    // without enumerating every image format.
    if extensions.is_empty() {
        return true;
    }

    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    let ext_lower = ext.to_ascii_lowercase();
    extensions
        .iter()
        .any(|allowed| allowed.to_ascii_lowercase() == ext_lower)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn config_with_dirs(root: &Path) -> Config {
        Config {
            include_dirs: vec![root.to_path_buf()],
            exclude_dirs: Vec::new(),
            extensions: Vec::new(),
            perceptual: None,
            fail_on_duplicates: false,
            ignore_cache: false,
            cache_dir: None,
        }
    }

    #[test]
    fn empty_include_dirs_returns_empty() {
        let config = Config {
            include_dirs: Vec::new(),
            ..Default::default()
        };
        assert!(discover_images(&config).is_empty());
    }

    #[test]
    fn nonexistent_include_dir_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            include_dirs: vec![dir.path().join("does-not-exist")],
            ..Default::default()
        };
        assert!(discover_images(&config).is_empty());
    }

    #[test]
    fn empty_directory_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_with_dirs(dir.path());
        assert!(discover_images(&config).is_empty());
    }

    #[test]
    fn discovers_files_in_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.png"), b"x").unwrap();
        fs::write(dir.path().join("b.jpg"), b"y").unwrap();

        let config = config_with_dirs(dir.path());
        let entries = discover_images(&config);

        assert_eq!(entries.len(), 2);
        let paths: Vec<_> = entries
            .iter()
            .map(|e| e.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(paths.contains(&"a.png".to_string()));
        assert!(paths.contains(&"b.jpg".to_string()));
    }

    #[test]
    fn exclude_dirs_skip_nested_folders() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(dir.path().join("a.png"), b"x").unwrap();
        fs::write(nested.join("b.png"), b"y").unwrap();

        let config = Config {
            exclude_dirs: vec![nested],
            ..config_with_dirs(dir.path())
        };
        let entries = discover_images(&config);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path.file_name().unwrap(), "a.png");
    }

    #[test]
    fn extension_filter_accepts_allowed_and_rejects_others() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.png"), b"x").unwrap();
        fs::write(dir.path().join("b.jpg"), b"y").unwrap();
        fs::write(dir.path().join("c.txt"), b"z").unwrap();

        let config = Config {
            extensions: vec!["png".to_string()],
            ..config_with_dirs(dir.path())
        };
        let entries = discover_images(&config);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path.file_name().unwrap(), "a.png");
    }

    #[test]
    fn extension_filter_is_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("lower.png"), b"x").unwrap();
        fs::write(dir.path().join("upper.PNG"), b"y").unwrap();

        let config = Config {
            extensions: vec!["png".to_string()],
            ..config_with_dirs(dir.path())
        };
        let entries = discover_images(&config);

        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn unsupported_extensions_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("image.png"), b"x").unwrap();
        fs::write(dir.path().join("notes.txt"), b"y").unwrap();
        fs::write(dir.path().join("noext"), b"z").unwrap();

        let config = Config {
            extensions: vec!["png".to_string(), "jpg".to_string()],
            ..config_with_dirs(dir.path())
        };
        let entries = discover_images(&config);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path.file_name().unwrap(), "image.png");
    }

    #[test]
    fn symlinks_are_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.png");
        let link = dir.path().join("link.png");
        fs::write(&target, b"x").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, &link).unwrap();

        let config = config_with_dirs(dir.path());
        let entries = discover_images(&config);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path.file_name().unwrap(), "target.png");
    }

    #[test]
    fn staged_path_kept_relative_when_inside_cwd() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("public").join("a.png");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, b"x").unwrap();

        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let result = std::panic::catch_unwind(|| {
            let config = config_with_dirs(Path::new("public"));
            let entries = discover_staged_images(&config, &[PathBuf::from("public/a.png")]);
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].path, PathBuf::from("public/a.png"));
        });
        std::env::set_current_dir(original).unwrap();
        result.unwrap();
    }

    #[test]
    fn staged_path_outside_cwd_kept_as_is() {
        let dir = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let file = other.path().join("a.png");
        fs::write(&file, b"x").unwrap();

        let config = config_with_dirs(dir.path());
        let entries = discover_staged_images(&config, &[file.clone()]);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, file);
    }

    #[test]
    fn file_metadata_reads_size_and_modified() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.png");
        fs::write(&file, b"hello").unwrap();

        let (size, modified) = file_metadata(&file).unwrap();
        assert_eq!(size, 5);
        assert!(modified > 0);
    }

    #[test]
    fn file_metadata_returns_none_for_missing_file() {
        let path = PathBuf::from("/definitely/missing/file.png");
        assert!(file_metadata(&path).is_none());
    }
}
