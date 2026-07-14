use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub include_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub exclude_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub extensions: Vec<String>,
}

impl Config {
    pub fn new(include_dirs: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        Self {
            include_dirs: include_dirs.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}
