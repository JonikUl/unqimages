use crate::perceptual::group_by_perceptual_hash;
use crate::{DuplicateGroup, DuplicateKind, ImageEntry, PerceptualConfig};
use std::collections::{HashMap, HashSet};
use std::io;

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

/// Combine exact-hash and optional perceptual groups.
///
/// Exact duplicates take precedence: any entry that is already part of an
/// exact duplicate group is excluded from perceptual grouping. This prevents
/// the same pair of images from appearing in both kinds of groups.
pub fn find_combined_duplicates(
    entries: Vec<ImageEntry>,
    perceptual: Option<&PerceptualConfig>,
) -> io::Result<Vec<DuplicateGroup>> {
    let exact_groups = group_by_file_hash(entries.clone());
    let exact_paths: HashSet<std::path::PathBuf> = exact_groups
        .iter()
        .flat_map(|g| &g.entries)
        .map(|e| e.path.clone())
        .collect();

    let mut combined = exact_groups;

    if let Some(config) = perceptual {
        if config.enabled {
            let candidates: Vec<ImageEntry> = entries
                .into_iter()
                .filter(|e| !exact_paths.contains(&e.path))
                .collect();
            let perceptual_groups = group_by_perceptual_hash(candidates, config.threshold)?;
            combined.extend(perceptual_groups);
        }
    }

    Ok(combined)
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

    fn entry_with_perceptual(
        path: &str,
        file_hash: &str,
        perceptual_hash: &str,
    ) -> ImageEntry {
        ImageEntry {
            path: PathBuf::from(path),
            size: 0,
            modified: 0,
            file_hash: Some(file_hash.to_string()),
            perceptual_hash: Some(perceptual_hash.to_string()),
        }
    }

    #[test]
    fn exact_groups_take_precedence_over_perceptual() {
        let entries = vec![
            entry_with_perceptual("a.png", "exact1", "p1"),
            entry_with_perceptual("b.png", "exact1", "p1"),
            entry_with_perceptual("c.png", "exact2", "p1"),
        ];

        let config = PerceptualConfig {
            enabled: true,
            threshold: 0,
        };
        let groups = find_combined_duplicates(entries, Some(&config)).unwrap();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].kind, DuplicateKind::Exact);

        let paths: Vec<_> = groups[0]
            .entries
            .iter()
            .map(|e| e.path.to_string_lossy().to_string())
            .collect();
        assert!(paths.contains(&"a.png".to_string()));
        assert!(paths.contains(&"b.png".to_string()));
        assert!(!paths.contains(&"c.png".to_string()));
    }
}
