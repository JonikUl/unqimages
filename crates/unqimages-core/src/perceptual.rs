use crate::{DuplicateGroup, DuplicateKind, ImageEntry};
use image_hasher::{HashAlg, HasherConfig, ImageHash};
use std::collections::HashMap;
use std::io;
use std::path::Path;

const HASH_SIZE: u32 = 8;

/// Compute a perceptual hash for an image file using the `image_hasher` crate.
///
/// Uses the Gradient (difference) hash algorithm at 8×8 resolution. Returns
/// `Ok(None)` for unsupported image formats; other decode failures are logged
/// and also return `Ok(None)` so discovery stays lenient.
pub fn compute_perceptual_hash(path: &Path) -> io::Result<Option<String>> {
    let image = match image::open(path) {
        Ok(img) => img,
        Err(image::ImageError::Unsupported(_)) => return Ok(None),
        Err(e) => {
            log::warn!("failed to decode image {}: {}", path.display(), e);
            return Ok(None);
        }
    };

    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Gradient)
        .hash_size(HASH_SIZE, HASH_SIZE)
        .to_hasher();

    Ok(Some(hasher.hash_image(&image).to_base64()))
}

pub fn hamming_distance(left: &str, right: &str) -> io::Result<u32> {
    let left = ImageHash::<Box<[u8]>>::from_base64(left).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid perceptual hash: {e:?}"),
        )
    })?;
    let right = ImageHash::<Box<[u8]>>::from_base64(right).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid perceptual hash: {e:?}"),
        )
    })?;
    Ok(left.dist(&right))
}

/// Group entries whose perceptual hashes are within `threshold` bits of each
/// other. Entries without a perceptual hash are ignored.
///
/// Uses connected components so that chains of similar images are placed in the
/// same group even if some pairwise distances exceed the threshold.
pub fn group_by_perceptual_hash(
    entries: Vec<ImageEntry>,
    threshold: u8,
) -> io::Result<Vec<DuplicateGroup>> {
    let entries: Vec<_> = entries
        .into_iter()
        .filter_map(|e| e.perceptual_hash.clone().map(|h| (e, h)))
        .collect();

    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let n = entries.len();
    let mut uf = UnionFind::new(n);

    for i in 0..n {
        for j in (i + 1)..n {
            let distance = hamming_distance(&entries[i].1, &entries[j].1)?;
            if distance <= threshold as u32 {
                uf.union(i, j);
            }
        }
    }

    let mut groups: HashMap<usize, Vec<ImageEntry>> = HashMap::new();
    let mut representative_hash: HashMap<usize, String> = HashMap::new();

    for (i, (entry, hash)) in entries.into_iter().enumerate() {
        let root = uf.find(i);
        groups.entry(root).or_default().push(entry);
        representative_hash.entry(root).or_insert(hash);
    }

    Ok(groups
        .into_iter()
        .filter(|(_, entries)| entries.len() > 1)
        .map(|(root, entries)| DuplicateGroup {
            hash: representative_hash[&root].clone(),
            kind: DuplicateKind::Perceptual,
            entries,
        })
        .collect())
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut current = x;
        while self.parent[current] != root {
            let next = self.parent[current];
            self.parent[current] = root;
            current = next;
        }
        root
    }

    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x == root_y {
            return;
        }
        match self.rank[root_x].cmp(&self.rank[root_y]) {
            std::cmp::Ordering::Less => self.parent[root_x] = root_y,
            std::cmp::Ordering::Greater => self.parent[root_y] = root_x,
            std::cmp::Ordering::Equal => {
                self.parent[root_y] = root_x;
                self.rank[root_x] += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;
    use std::io::Write;
    use std::path::PathBuf;

    fn save_image(path: &Path, image: &RgbImage) {
        image.save(path).unwrap();
    }

    fn gradient_image() -> RgbImage {
        let mut img = RgbImage::new(64, 64);
        for (x, _y, pixel) in img.enumerate_pixels_mut() {
            let v = (x * 4) as u8;
            *pixel = image::Rgb([v, v, v]);
        }
        img
    }

    fn noise_image() -> RgbImage {
        let mut img = RgbImage::new(64, 64);
        for (x, _y, pixel) in img.enumerate_pixels_mut() {
            let v = if x % 2 == 0 { 255 } else { 0 };
            *pixel = image::Rgb([v, v, v]);
        }
        img
    }

    fn entry(path: &str, hash: &str) -> ImageEntry {
        ImageEntry {
            path: PathBuf::from(path),
            size: 0,
            modified: 0,
            file_hash: None,
            perceptual_hash: Some(hash.to_string()),
        }
    }

    fn hash_from_bytes(bytes: &[u8]) -> String {
        ImageHash::<Box<[u8]>>::from_bytes(bytes).unwrap().to_base64()
    }

    #[test]
    fn identical_images_have_zero_distance() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        save_image(&a, &gradient_image());
        save_image(&b, &gradient_image());

        let ha = compute_perceptual_hash(&a).unwrap().unwrap();
        let hb = compute_perceptual_hash(&b).unwrap().unwrap();
        assert_eq!(hamming_distance(&ha, &hb).unwrap(), 0);
    }

    #[test]
    fn different_images_have_non_zero_distance() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        save_image(&a, &gradient_image());
        save_image(&b, &noise_image());

        let ha = compute_perceptual_hash(&a).unwrap().unwrap();
        let hb = compute_perceptual_hash(&b).unwrap().unwrap();
        assert!(hamming_distance(&ha, &hb).unwrap() > 0);
    }

    #[test]
    fn unsupported_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("not-an-image.txt");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"hello")
            .unwrap();

        assert_eq!(compute_perceptual_hash(&path).unwrap(), None);
    }

    #[test]
    fn threshold_zero_groups_only_identical_perceptual_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        let c = dir.path().join("c.png");
        save_image(&a, &gradient_image());
        save_image(&b, &gradient_image());
        save_image(&c, &noise_image());

        let ha = compute_perceptual_hash(&a).unwrap().unwrap();
        let hb = compute_perceptual_hash(&b).unwrap().unwrap();
        let hc = compute_perceptual_hash(&c).unwrap().unwrap();

        let entries = vec![
            entry("a.png", &ha),
            entry("b.png", &hb),
            entry("c.png", &hc),
        ];

        let groups = group_by_perceptual_hash(entries, 0).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].kind, DuplicateKind::Perceptual);
        assert_eq!(groups[0].entries.len(), 2);
    }

    #[test]
    fn max_threshold_connects_any_two_images() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        save_image(&a, &gradient_image());
        save_image(&b, &noise_image());

        let ha = compute_perceptual_hash(&a).unwrap().unwrap();
        let hb = compute_perceptual_hash(&b).unwrap().unwrap();

        let entries = vec![entry("a.png", &ha), entry("b.png", &hb)];
        let groups = group_by_perceptual_hash(entries, 64).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].entries.len(), 2);
    }

    #[test]
    fn transitive_grouping_via_connected_components() {
        // A-B differ by 8 bits, B-C differ by 8 bits, A-C differ by 16 bits.
        let ha = hash_from_bytes(&[0b0000_0000; 8]);
        let hb = hash_from_bytes(&[0b0000_0001; 8]);
        let hc = hash_from_bytes(&[0b0000_0011; 8]);

        assert_eq!(hamming_distance(&ha, &hb).unwrap(), 8);
        assert_eq!(hamming_distance(&hb, &hc).unwrap(), 8);
        assert_eq!(hamming_distance(&ha, &hc).unwrap(), 16);

        let entries = vec![
            entry("a.png", &ha),
            entry("b.png", &hb),
            entry("c.png", &hc),
        ];

        // Threshold 10 connects A-B and B-C, so all three end up in one group.
        let groups = group_by_perceptual_hash(entries.clone(), 10).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].entries.len(), 3);

        // Threshold 7 connects nothing.
        let groups = group_by_perceptual_hash(entries, 7).unwrap();
        assert!(groups.is_empty());
    }
}
