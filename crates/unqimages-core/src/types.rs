use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub perceptual: bool,
    pub cache_dir: Option<PathBuf>,
    #[serde(default = "default_fail_on_duplicates")]
    pub fail_on_duplicates: bool,
}

fn default_fail_on_duplicates() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntry {
    pub path: PathBuf,
    pub size: u64,
    pub exact_hash: String,
    pub perceptual_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub kind: DuplicateKind,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateKind {
    Exact,
    Perceptual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub path: PathBuf,
    pub modified: u64,
    pub exact_hash: String,
    pub perceptual_hash: Option<String>,
}
