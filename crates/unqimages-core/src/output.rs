use unqimages_core::{DuplicateGroup, DuplicateKind};

#[cfg(test)]
use unqimages_core::ImageEntry;
use serde::Serialize;
use std::io::{self, Write};

/// stdout payload; keep the schema stable so the TypeScript wrapper can parse it.
#[derive(Debug, Serialize)]
pub struct CliOutput {
    pub duplicates: Vec<DuplicateGroup>,
    pub scanned: usize,
    pub elapsed_ms: u64,
    pub used_cache: bool,
}

impl CliOutput {
    pub fn to_json(&self) -> io::Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

pub fn print_json(output: &CliOutput, writer: &mut dyn Write) -> io::Result<()> {
    let json = output.to_json()?;
    writer.write_all(json.as_bytes())?;
    writer.write_all(b"\n")?;
    Ok(())
}

pub fn print_table(output: &CliOutput, writer: &mut dyn Write) -> io::Result<()> {
    let duplicate_count: usize = output.duplicates.iter().map(|g| g.entries.len()).sum();
    let cache_note = if output.used_cache { " [used cache]" } else { "" };
    writeln!(
        writer,
        "Found {} duplicate group(s) ({} file(s)) in {} scanned file(s) ({} ms){}",
        output.duplicates.len(),
        duplicate_count,
        output.scanned,
        output.elapsed_ms,
        cache_note
    )?;

    if output.duplicates.is_empty() {
        return Ok(());
    }

    writeln!(writer)?;

    for group in &output.duplicates {
        let kind_label = match group.kind {
            DuplicateKind::Exact => "Exact",
            DuplicateKind::Perceptual => "Perceptual",
        };
        writeln!(writer, "{}: {}", kind_label, group.hash)?;
        for entry in &group.entries {
            writeln!(writer, "  - {}", entry.path.display())?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn group(kind: DuplicateKind) -> DuplicateGroup {
        DuplicateGroup {
            hash: "abc123".to_string(),
            kind,
            entries: vec![
                ImageEntry {
                    path: PathBuf::from("a.png"),
                    size: 100,
                    modified: 0,
                    file_hash: None,
                    perceptual_hash: None,
                },
                ImageEntry {
                    path: PathBuf::from("b.png"),
                    size: 100,
                    modified: 0,
                    file_hash: None,
                    perceptual_hash: None,
                },
            ],
        }
    }

    #[test]
    fn json_output_serializes() {
        let output = CliOutput {
            duplicates: vec![group(DuplicateKind::Exact)],
            scanned: 10,
            elapsed_ms: 42,
            used_cache: false,
        };
        let json = output.to_json().unwrap();
        assert!(json.contains("\"scanned\": 10"));
        assert!(json.contains("\"elapsed_ms\": 42"));
        assert!(json.contains("\"hash\": \"abc123\""));
    }

    #[test]
    fn json_output_prints_with_trailing_newline() {
        let output = CliOutput {
            duplicates: vec![group(DuplicateKind::Exact)],
            scanned: 10,
            elapsed_ms: 42,
            used_cache: false,
        };
        let mut buf = Vec::new();
        print_json(&output, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("\"hash\": \"abc123\""));
        assert!(text.ends_with('\n'));
    }

    #[test]
    fn table_output_contains_paths_and_summary() {
        let output = CliOutput {
            duplicates: vec![group(DuplicateKind::Exact)],
            scanned: 10,
            elapsed_ms: 42,
            used_cache: false,
        };
        let mut buf = Vec::new();
        print_table(&output, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Found 1 duplicate group(s) (2 file(s)) in 10 scanned file(s) (42 ms)"));
        assert!(text.contains("Exact: abc123"));
        assert!(text.contains("a.png"));
        assert!(text.contains("b.png"));
    }

    #[test]
    fn table_output_labels_perceptual_groups() {
        let output = CliOutput {
            duplicates: vec![group(DuplicateKind::Perceptual)],
            scanned: 10,
            elapsed_ms: 42,
            used_cache: false,
        };
        let mut buf = Vec::new();
        print_table(&output, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Perceptual: abc123"));
        assert!(!text.contains("Exact:"));
    }

    #[test]
    fn table_output_for_multiple_groups() {
        let exact = group(DuplicateKind::Exact);
        let perceptual = DuplicateGroup {
            hash: "def456".to_string(),
            kind: DuplicateKind::Perceptual,
            entries: vec![
                ImageEntry {
                    path: PathBuf::from("c.png"),
                    size: 100,
                    modified: 0,
                    file_hash: None,
                    perceptual_hash: None,
                },
                ImageEntry {
                    path: PathBuf::from("d.png"),
                    size: 100,
                    modified: 0,
                    file_hash: None,
                    perceptual_hash: None,
                },
            ],
        };
        let output = CliOutput {
            duplicates: vec![exact, perceptual],
            scanned: 10,
            elapsed_ms: 42,
            used_cache: false,
        };
        let mut buf = Vec::new();
        print_table(&output, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Found 2 duplicate group(s) (4 file(s)) in 10 scanned file(s) (42 ms)"));
        assert!(text.contains("Exact: abc123"));
        assert!(text.contains("Perceptual: def456"));
    }

    #[test]
    fn table_output_for_empty_duplicates() {
        let output = CliOutput {
            duplicates: Vec::new(),
            scanned: 5,
            elapsed_ms: 7,
            used_cache: false,
        };
        let mut buf = Vec::new();
        print_table(&output, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Found 0 duplicate group(s) (0 file(s)) in 5 scanned file(s) (7 ms)"));
        assert!(!text.contains("Exact:"));
    }

    #[test]
    fn table_output_shows_cache_note_when_used() {
        let output = CliOutput {
            duplicates: Vec::new(),
            scanned: 5,
            elapsed_ms: 7,
            used_cache: true,
        };
        let mut buf = Vec::new();
        print_table(&output, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("[used cache]"));
    }
}
