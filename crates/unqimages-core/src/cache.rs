use crate::CacheEntry;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const CACHE_VERSION: u32 = 1;
const CURRENT_ALGORITHM_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheFile {
    version: u32,
    entries: Vec<CacheEntry>,
}

#[derive(Debug)]
enum CacheInner {
    Active {
        path: PathBuf,
        lock_file: File,
        entries: HashMap<PathBuf, CacheEntry>,
    },
    Disabled,
}

/// Disk-backed cache for file hashes.
///
/// Loading acquires a shared advisory lock on a sidecar lock file; saving
/// upgrades to an exclusive lock, re-reads the current cache to merge any
/// concurrent updates, then writes atomically via a temporary file + rename.
#[derive(Debug)]
pub struct Cache {
    inner: CacheInner,
    used: bool,
    dirty: bool,
}

impl Cache {
    /// Returns a cache that never reads or writes disk. Use when caching is
    /// disabled by configuration or when the lock cannot be acquired.
    pub fn disabled() -> Self {
        Self {
            inner: CacheInner::Disabled,
            used: false,
            dirty: false,
        }
    }

    /// Loads the cache at `path`, creating the parent directory and the cache
    /// lock file as needed. If the cache file is corrupted or has an unexpected
    /// version, it is ignored with a warning and rebuilt on save.
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                log::warn!("failed to create cache directory: {}", e);
                return Self::disabled();
            }
        }

        let lock_path = lock_path(&path);
        let lock_file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
        {
            Ok(f) => f,
            Err(e) => {
                log::warn!("failed to open cache lock file: {}", e);
                return Self::disabled();
            }
        };

        if let Err(e) = lock_file.try_lock_shared() {
            log::warn!("cache is locked by another process: {}", e);
            return Self::disabled();
        }

        let entries = match read_cache_file(&path) {
            Ok(entries) => entries,
            Err(e) => {
                log::warn!("failed to read cache file, rebuilding: {}", e);
                HashMap::new()
            }
        };

        Self {
            inner: CacheInner::Active {
                path,
                lock_file,
                entries,
            },
            used: false,
            dirty: false,
        }
    }

    /// Looks up a cache entry. The entry is considered valid only when `size`,
    /// `modified` and `algorithm_version` all match the current expectations.
    pub fn get(&self, path: &Path, size: u64, modified: u64) -> Option<&CacheEntry> {
        let entry = self.entries()?.get(path)?;
        if entry.size == size
            && entry.modified == modified
            && entry.algorithm_version == CURRENT_ALGORITHM_VERSION
        {
            Some(entry)
        } else {
            None
        }
    }

    pub fn insert(&mut self, entry: CacheEntry) {
        if let CacheInner::Active { entries, .. } = &mut self.inner {
            entries.insert(entry.path.clone(), entry);
            self.dirty = true;
        }
    }

    /// Marks that at least one cache lookup returned a valid entry.
    pub fn mark_used(&mut self) {
        self.used = true;
    }

    /// Writes the cache back to disk atomically if it has been modified.
    ///
    /// Uses a process-unique temporary file to avoid races with other writers,
    /// and keeps the exclusive lock on the sidecar file until the cache is
    /// dropped so the lock file remains protected for the cache lifetime.
    pub fn save(&mut self) -> io::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let CacheInner::Active {
            path,
            lock_file,
            entries,
        } = &mut self.inner
        else {
            return Ok(());
        };

        lock_file.lock_exclusive().map_err(|e| {
            io::Error::other(format!("failed to acquire exclusive cache lock: {}", e))
        })?;

        // Merge with any entries written by other processes since load,
        // preferring the most complete set of hashes for each path.
        let mut merged = entries.clone();
        if let Ok(existing) = read_cache_file(path) {
            for (key, disk) in existing {
                merged
                    .entry(key)
                    .and_modify(|mem| merge_entry(mem, &disk))
                    .or_insert(disk);
            }
        }

        let cache_file = CacheFile {
            version: CACHE_VERSION,
            entries: merged.into_values().collect(),
        };

        let json = serde_json::to_string_pretty(&cache_file)?;
        let temp_path = unique_temp_path(path);
        {
            let mut temp = File::create(&temp_path)?;
            temp.write_all(json.as_bytes())?;
            temp.sync_all()?;
        }
        fs::rename(&temp_path, path)?;

        self.dirty = false;
        Ok(())
    }

    pub fn used(&self) -> bool {
        self.used
    }

    fn entries(&self) -> Option<&HashMap<PathBuf, CacheEntry>> {
        match &self.inner {
            CacheInner::Active { entries, .. } => Some(entries),
            CacheInner::Disabled => None,
        }
    }
}

fn merge_entry(target: &mut CacheEntry, disk: &CacheEntry) {
    if target.perceptual_hash.is_none() && disk.perceptual_hash.is_some() {
        target.perceptual_hash.clone_from(&disk.perceptual_hash);
    }
    if target.file_hash.is_empty() && !disk.file_hash.is_empty() {
        target.file_hash.clone_from(&disk.file_hash);
    }
}

fn lock_path(cache_path: &Path) -> PathBuf {
    cache_path.with_extension("json.lock")
}

fn unique_temp_path(cache_path: &Path) -> PathBuf {
    let pid = std::process::id();
    let file_name = format!(
        "{}.tmp.{}",
        cache_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("cache"),
        pid
    );
    cache_path.with_file_name(file_name)
}

fn read_cache_file(path: &Path) -> io::Result<HashMap<PathBuf, CacheEntry>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let cache: CacheFile = serde_json::from_str(&contents)?;
    if cache.version != CACHE_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported cache version {}", cache.version),
        ));
    }
    Ok(cache
        .entries
        .into_iter()
        .map(|e| (e.path.clone(), e))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_cache() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.json");
        (dir, path)
    }

    fn entry(path: &str, size: u64, modified: u64, file_hash: &str) -> CacheEntry {
        CacheEntry {
            path: PathBuf::from(path),
            size,
            modified,
            file_hash: file_hash.to_string(),
            perceptual_hash: None,
            algorithm_version: CURRENT_ALGORITHM_VERSION,
        }
    }

    #[test]
    fn empty_cache_round_trip() {
        let (_dir, path) = temp_cache();
        let mut cache = Cache::load(&path);
        cache.save().unwrap();
        assert!(Cache::load(&path).get(Path::new("missing"), 1, 1).is_none());
    }

    #[test]
    fn stores_and_retrieves_entry() {
        let (_dir, path) = temp_cache();
        {
            let mut cache = Cache::load(&path);
            cache.insert(entry("a.png", 100, 200, "hash1"));
            cache.save().unwrap();
        }

        let cache = Cache::load(&path);
        let got = cache.get(Path::new("a.png"), 100, 200).unwrap();
        assert_eq!(got.file_hash, "hash1");
    }

    #[test]
    fn invalidates_on_size_mismatch() {
        let (_dir, path) = temp_cache();
        {
            let mut cache = Cache::load(&path);
            cache.insert(entry("a.png", 100, 200, "hash1"));
            cache.save().unwrap();
        }

        let cache = Cache::load(&path);
        assert!(cache.get(Path::new("a.png"), 999, 200).is_none());
    }

    #[test]
    fn invalidates_on_modified_mismatch() {
        let (_dir, path) = temp_cache();
        {
            let mut cache = Cache::load(&path);
            cache.insert(entry("a.png", 100, 200, "hash1"));
            cache.save().unwrap();
        }

        let cache = Cache::load(&path);
        assert!(cache.get(Path::new("a.png"), 100, 999).is_none());
    }

    #[test]
    fn invalidates_on_algorithm_version_mismatch() {
        let (_dir, path) = temp_cache();
        {
            let mut cache = Cache::load(&path);
            let mut stale = entry("a.png", 100, 200, "hash1");
            stale.algorithm_version = 0;
            cache.insert(stale);
            cache.save().unwrap();
        }

        let cache = Cache::load(&path);
        assert!(cache.get(Path::new("a.png"), 100, 200).is_none());
    }

    #[test]
    fn corrupted_cache_is_ignored() {
        let (_dir, path) = temp_cache();
        {
            let mut file = File::create(&path).unwrap();
            file.write_all(b"not json").unwrap();
        }

        {
            let cache = Cache::load(&path);
            assert!(cache.get(Path::new("a.png"), 1, 1).is_none());
        }

        // A subsequent save should rebuild a valid file.
        {
            let mut cache = Cache::load(&path);
            cache.insert(entry("a.png", 1, 1, "hash"));
            cache.save().unwrap();
        }
        assert!(Cache::load(&path).get(Path::new("a.png"), 1, 1).is_some());
    }

    #[test]
    fn atomic_write_uses_unique_temp_file() {
        let (dir, path) = temp_cache();
        {
            let mut cache = Cache::load(&path);
            cache.insert(entry("a.png", 1, 1, "hash"));
            cache.save().unwrap();
        }

        let mut tmp_seen = false;
        for result in fs::read_dir(dir.path()).unwrap() {
            let name = result.unwrap().file_name();
            if name.to_string_lossy().contains(".tmp.") {
                tmp_seen = true;
                break;
            }
        }
        assert!(!tmp_seen);
    }

    #[test]
    fn disabled_cache_swallows_operations_and_does_not_write() {
        let (dir, path) = temp_cache();

        let mut cache = Cache::disabled();
        cache.insert(entry("a.png", 1, 1, "hash"));
        cache.save().unwrap();

        assert!(!cache.used());
        assert!(!path.exists());
        assert!(dir.path().read_dir().unwrap().next().is_none());
    }

    #[test]
    fn merge_preserves_disk_perceptual_hash() {
        let (_dir, path) = temp_cache();
        {
            let mut cache = Cache::load(&path);
            let mut disk = entry("a.png", 100, 200, "disk_hash");
            disk.perceptual_hash = Some("perceptual".to_string());
            cache.insert(disk);
            cache.save().unwrap();
        }

        {
            let mut cache = Cache::load(&path);
            // In-memory entry has no perceptual hash; disk entry does.
            cache.insert(CacheEntry {
                path: PathBuf::from("a.png"),
                size: 100,
                modified: 200,
                file_hash: "mem_hash".to_string(),
                perceptual_hash: None,
                algorithm_version: CURRENT_ALGORITHM_VERSION,
            });
            cache.save().unwrap();
        }

        let cache = Cache::load(&path);
        let got = cache.get(Path::new("a.png"), 100, 200).unwrap();
        assert_eq!(got.file_hash, "mem_hash");
        assert_eq!(got.perceptual_hash, Some("perceptual".to_string()));
    }

    #[test]
    fn interrupted_write_is_recovered_by_next_save() {
        let (_dir, path) = temp_cache();

        {
            let mut cache = Cache::load(&path);
            cache.insert(entry("a.png", 1, 1, "hash-a"));
            cache.save().unwrap();
        }

        // Simulate an interrupted write from the same process: a stale temp
        // file exists but cache.json is still valid.
        let temp_path = unique_temp_path(&path);
        {
            let mut stale = File::create(&temp_path).unwrap();
            stale.write_all(b"partial json {").unwrap();
        }

        {
            let mut cache = Cache::load(&path);
            cache.insert(entry("b.png", 2, 2, "hash-b"));
            cache.save().unwrap();
        }

        assert!(!temp_path.exists());

        let cache = Cache::load(&path);
        assert!(cache.get(Path::new("a.png"), 1, 1).is_some());
        assert!(cache.get(Path::new("b.png"), 2, 2).is_some());
    }
}
