use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
    pub file_hash: Option<String>,
    pub perceptual_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub kind: DuplicateKind,
    pub entries: Vec<ImageEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateKind {
    Exact,
    Perceptual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64,
    pub file_hash: String,
    pub perceptual_hash: Option<String>,
    /// `serde(default)` keeps older cache files readable; entries with a stale
    /// version are invalidated at lookup time.
    #[serde(default)]
    pub algorithm_version: u32,
}

/// Returned together so the CLI can report scanned counts without a second walk.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub groups: Vec<DuplicateGroup>,
    pub scanned: usize,
    pub used_cache: bool,
}
