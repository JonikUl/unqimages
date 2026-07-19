pub mod cache;
pub mod config;
pub mod discovery;
pub mod duplicates;
pub mod hash;
pub mod perceptual;
pub mod types;

pub use cache::*;
pub use config::*;
pub use discovery::*;
pub use duplicates::*;
pub use hash::*;
pub use perceptual::*;
pub use types::*;

use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

/// Default cache directory relative to the current working directory.
pub const DEFAULT_CACHE_DIR: &str = "node_modules/.cache/unqimages";

/// Current version of the hashing algorithms used to produce cached hashes.
/// Bumped whenever the exact-hash or perceptual-hash implementation changes.
pub const CACHE_ALGORITHM_VERSION: u32 = 1;

/// Ignores perceptual settings so callers get a fast, deterministic check.
pub fn find_exact_duplicates(config: &Config) -> io::Result<ScanResult> {
    find_duplicates_impl(config, false)
}

pub fn find_duplicates(config: &Config) -> io::Result<ScanResult> {
    find_duplicates_impl(config, true)
}

/// Checks only the provided staged paths as "new" files, while still scanning
/// the rest of the project so duplicates against existing files are detected.
/// Returned duplicate groups are limited to those that involve at least one
/// staged file, and `scanned` reflects the number of staged image files.
pub fn find_duplicates_with_staged(
    config: &Config,
    staged_paths: &[PathBuf],
) -> io::Result<ScanResult> {
    find_duplicates_staged_impl(config, staged_paths, true)
}

fn find_duplicates_impl(config: &Config, include_perceptual: bool) -> io::Result<ScanResult> {
    let entries = discover_images(config);
    let scanned = entries.len();
    process_entries(config, entries, scanned, include_perceptual)
}

fn find_duplicates_staged_impl(
    config: &Config,
    staged_paths: &[PathBuf],
    include_perceptual: bool,
) -> io::Result<ScanResult> {
    // Discover staged files first so we can use their normalized paths when
    // filtering both the existing-file set and the resulting duplicate groups.
    let mut staged = discover_staged_images(config, staged_paths);
    let scanned = staged.len();

    if scanned == 0 {
        return Ok(ScanResult {
            groups: Vec::new(),
            scanned: 0,
            used_cache: false,
        });
    }

    let staged_set: HashSet<PathBuf> = staged.iter().map(|entry| entry.path.clone()).collect();

    // Use canonical paths for deduplication so the same file is not processed
    // twice when include_dirs are absolute and staged paths are relative (or
    // vice versa).
    let staged_canonical: HashSet<PathBuf> = staged
        .iter()
        .filter_map(|entry| std::fs::canonicalize(&entry.path).ok())
        .collect();

    let mut existing = discover_images(config);
    existing.retain(|entry| {
        if staged_set.contains(&entry.path) {
            return false;
        }
        if let Ok(canonical) = std::fs::canonicalize(&entry.path) {
            if staged_canonical.contains(&canonical) {
                return false;
            }
        }
        true
    });

    let mut entries = Vec::with_capacity(existing.len() + staged.len());
    entries.append(&mut existing);
    entries.append(&mut staged);

    let mut result = process_entries(config, entries, scanned, include_perceptual)?;

    result.groups.retain(|group| {
        group
            .entries
            .iter()
            .any(|entry| staged_set.contains(&entry.path))
    });

    Ok(result)
}

fn process_entries(
    config: &Config,
    mut entries: Vec<ImageEntry>,
    scanned: usize,
    include_perceptual: bool,
) -> io::Result<ScanResult> {
    let cache_path = cache_path(config);
    let mut cache = if config.ignore_cache {
        log::info!("cache ignored by configuration");
        Cache::disabled()
    } else {
        Cache::load(&cache_path)
    };

    for entry in &mut entries {
        if let Some(cached) = cache.get(&entry.path, entry.size, entry.modified) {
            entry.file_hash = Some(cached.file_hash.clone());
            cache.mark_used();
        } else {
            let file_hash = hash_file(&entry.path)?;
            cache.insert(CacheEntry {
                path: entry.path.clone(),
                size: entry.size,
                modified: entry.modified,
                file_hash: file_hash.clone(),
                perceptual_hash: None,
                algorithm_version: CACHE_ALGORITHM_VERSION,
            });
            entry.file_hash = Some(file_hash);
        }
    }

    let perceptual = if include_perceptual {
        config.perceptual.as_ref()
    } else {
        None
    };

    if perceptual.map(|p| p.enabled).unwrap_or(false) {
        for entry in &mut entries {
            if let Some(cached) = cache.get(&entry.path, entry.size, entry.modified) {
                if cached.perceptual_hash.is_some() {
                    entry.perceptual_hash = cached.perceptual_hash.clone();
                    continue;
                }
            }

            let perceptual_hash = compute_perceptual_hash(&entry.path)?;
            // Preserve the exact hash together with the new perceptual hash so
            // future cache hits can reuse both without re-reading the file.
            if let Some(file_hash) = &entry.file_hash {
                cache.insert(CacheEntry {
                    path: entry.path.clone(),
                    size: entry.size,
                    modified: entry.modified,
                    file_hash: file_hash.clone(),
                    perceptual_hash: perceptual_hash.clone(),
                    algorithm_version: CACHE_ALGORITHM_VERSION,
                });
            }
            entry.perceptual_hash = perceptual_hash;
        }
    }

    let groups = find_combined_duplicates(entries, perceptual)?;
    let used_cache = cache.used();
    if let Err(e) = cache.save() {
        log::warn!("failed to write cache, continuing without it: {}", e);
    }
    Ok(ScanResult {
        groups,
        scanned,
        used_cache,
    })
}

fn cache_path(config: &Config) -> PathBuf {
    config
        .cache_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CACHE_DIR))
        .join("cache.json")
}
