use serde::{de, Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

const MAX_PERCEPTUAL_THRESHOLD: u8 = 64; // 8×8 perceptual hash = 64 bits

/// Defaults are chosen so the binary works on first run without project setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_include_dirs")]
    pub include_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub exclude_dirs: Vec<PathBuf>,
    #[serde(default = "default_extensions")]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub perceptual: Option<PerceptualConfig>,
    #[serde(default)]
    pub fail_on_duplicates: bool,
    #[serde(default)]
    pub ignore_cache: bool,
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            include_dirs: default_include_dirs(),
            exclude_dirs: Vec::new(),
            extensions: default_extensions(),
            perceptual: None,
            fail_on_duplicates: false,
            ignore_cache: false,
            cache_dir: None,
        }
    }
}

fn default_include_dirs() -> Vec<PathBuf> {
    vec![PathBuf::from("src/assets"), PathBuf::from("public")]
}

fn default_extensions() -> Vec<String> {
    ["png", "jpg", "jpeg", "gif", "webp", "svg", "ico"]
        .into_iter()
        .map(String::from)
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptualConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_threshold", deserialize_with = "deserialize_threshold")]
    pub threshold: u8,
}

impl Default for PerceptualConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: default_threshold(),
        }
    }
}

fn default_threshold() -> u8 {
    10
}

fn deserialize_threshold<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u8::deserialize(deserializer)?;
    if value > MAX_PERCEPTUAL_THRESHOLD {
        return Err(de::Error::custom(format!(
            "perceptual threshold must be <= {MAX_PERCEPTUAL_THRESHOLD}, got {value}"
        )));
    }
    Ok(value)
}

impl Config {
    pub fn new(include_dirs: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        Self {
            include_dirs: include_dirs.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_perceptual_threshold_is_ten() {
        let config = PerceptualConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.threshold, 10);
    }

    #[test]
    fn threshold_within_range_deserializes() {
        let json = r#"{"enabled": true, "threshold": 42}"#;
        let config: PerceptualConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.threshold, 42);
    }

    #[test]
    fn threshold_above_max_rejects() {
        let json = r#"{"enabled": true, "threshold": 65}"#;
        let result: Result<PerceptualConfig, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
