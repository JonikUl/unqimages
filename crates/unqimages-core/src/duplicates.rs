use crate::{DuplicateGroup, DuplicateKind, ImageEntry};
use std::collections::HashMap;

/// Entries with no hash are ignored. Each returned group contains at least
/// two entries and is tagged with `DuplicateKind::Exact`.
pub fn group_by_file_hash(entries: Vec<ImageEntry>) -> Vec<DuplicateGroup> {
    let mut groups: HashMap<String, Vec<ImageEntry>> = HashMap::new();

    for entry in entries {
        if let Some(hash) = &entry.file_hash {
            groups.entry(hash.clone()).or_default().push(entry);
        }
    }

    groups
        .into_iter()
        .filter(|(_, entries)| entries.len() > 1)
        .map(|(hash, entries)| DuplicateGroup {
            hash,
            kind: DuplicateKind::Exact,
            entries,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn entry(path: &str, hash: &str) -> ImageEntry {
        ImageEntry {
            path: PathBuf::from(path),
            size: 0,
            modified: 0,
            file_hash: Some(hash.to_string()),
            perceptual_hash: None,
        }
    }

    #[test]
    fn groups_entries_with_matching_hash() {
        let entries = vec![
            entry("a.png", "h1"),
            entry("b.png", "h1"),
            entry("c.png", "h2"),
        ];

        let groups = group_by_file_hash(entries);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].hash, "h1");
        assert_eq!(groups[0].entries.len(), 2);
    }

    #[test]
    fn ignores_unique_and_missing_hashes() {
        let entries = vec![
            entry("a.png", "h1"),
            entry("b.png", "h2"),
            ImageEntry {
                path: PathBuf::from("c.png"),
                size: 0,
                modified: 0,
                file_hash: None,
                perceptual_hash: None,
            },
        ];

        let groups = group_by_file_hash(entries);
        assert!(groups.is_empty());
    }
}
