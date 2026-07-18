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

            let Ok(metadata) = fs::metadata(path) else { continue };
            let size = metadata.len();
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_nanos() as u64)
                // A pre-1970 mtime is treated as epoch to keep the cache schema
                // simple and monotonic.
                .unwrap_or(0);

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
