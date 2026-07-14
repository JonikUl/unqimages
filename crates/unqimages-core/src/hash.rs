use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

/// xxHash3 is used because it is much faster than SHA-256 for this workload
/// while keeping the collision probability negligible for exact-duplicate detection.
pub fn hash_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Xxhash3Adapter::default();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finish())
}

#[derive(Default)]
struct Xxhash3Adapter {
    buffer: Vec<u8>,
}

impl Xxhash3Adapter {
    fn update(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    fn finish(&self) -> String {
        format!("{:016x}", xxh3_64(&self.buffer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn hash_is_stable_for_same_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"hello")
            .unwrap();

        let first = hash_file(&path).unwrap();
        let second = hash_file(&path).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 16);
    }

    #[test]
    fn different_contents_yield_different_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        std::fs::File::create(&a).unwrap().write_all(b"a").unwrap();
        std::fs::File::create(&b).unwrap().write_all(b"b").unwrap();

        assert_ne!(hash_file(&a).unwrap(), hash_file(&b).unwrap());
    }
}
