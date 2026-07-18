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

fn find_duplicates_impl(
    config: &Config,
    include_perceptual: bool,
) -> io::Result<ScanResult> {
    // `find_exact_duplicates` must ignore perceptual settings even if the user
    // enabled them in the config, so the flag is passed explicitly.
    let mut entries = discover_images(config);
    let scanned = entries.len();

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
