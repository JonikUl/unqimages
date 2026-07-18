use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, RgbImage};
use std::fs;
use std::path::Path;
use unqimages_core::{find_duplicates, Config, DuplicateKind, PerceptualConfig};

fn save_image(path: &Path, image: &RgbImage) {
    image.save(path).unwrap();
}

fn gradient_image() -> RgbImage {
    let mut img = RgbImage::new(128, 128);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let v = ((x + y) / 2) as u8;
        *pixel = image::Rgb([v, v, v]);
    }
    img
}

fn noise_image() -> RgbImage {
    let mut img = RgbImage::new(128, 128);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let v = ((x * 2 + y * 3) % 256) as u8;
        *pixel = image::Rgb([v, 0, 255 - v]);
    }
    img
}

fn write_jpeg_copy(path: &Path, image: &RgbImage, quality: u8) {
    let dynamic = DynamicImage::ImageRgb8(image.clone());
    let mut buffer = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, quality);
    encoder.encode_image(&dynamic).unwrap();
    fs::write(path, &buffer).unwrap();
}

fn config_with_perceptual(root: &Path, enabled: bool, threshold: u8) -> Config {
    Config {
        include_dirs: vec![root.to_path_buf()],
        exclude_dirs: vec![],
        extensions: vec![],
        perceptual: Some(PerceptualConfig {
            enabled,
            threshold,
        }),
        fail_on_duplicates: false,
        cache_dir: Some(root.join(".cache")),
        ..Default::default()
    }
}

#[test]
fn perceptual_enabled_finds_compressed_copy() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let a = root.join("original.png");
    let b = root.join("compressed.jpg");
    let c = root.join("other.png");

    save_image(&a, &gradient_image());
    write_jpeg_copy(&b, &gradient_image(), 80);
    save_image(&c, &noise_image());

    let config = config_with_perceptual(root, true, 10);
    let result = find_duplicates(&config).unwrap();

    assert_eq!(result.groups.len(), 1);
    assert_eq!(result.groups[0].kind, DuplicateKind::Perceptual);

    let paths: Vec<_> = result.groups[0]
        .entries
        .iter()
        .map(|e| e.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();

    assert!(paths.contains(&"original.png".to_string()));
    assert!(paths.contains(&"compressed.jpg".to_string()));
    assert!(!paths.contains(&"other.png".to_string()));
}

#[test]
fn perceptual_disabled_still_finds_exact_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let a = root.join("original.png");
    let b = root.join("copy.png");
    let c = root.join("compressed.jpg");

    save_image(&a, &gradient_image());
    save_image(&b, &gradient_image());
    write_jpeg_copy(&c, &gradient_image(), 80);

    let config = config_with_perceptual(root, false, 10);
    let result = find_duplicates(&config).unwrap();

    assert_eq!(result.groups.len(), 1);
    assert_eq!(result.groups[0].kind, DuplicateKind::Exact);
    assert_eq!(result.groups[0].entries.len(), 2);

    let paths: Vec<_> = result.groups[0]
        .entries
        .iter()
        .map(|e| e.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(paths.contains(&"original.png".to_string()));
    assert!(paths.contains(&"copy.png".to_string()));
    assert!(!paths.contains(&"compressed.jpg".to_string()));
}

#[test]
fn threshold_zero_rejects_similar_but_not_identical_images() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let a = root.join("original.png");
    let b = root.join("other.png");

    save_image(&a, &gradient_image());
    save_image(&b, &noise_image());

    let config = config_with_perceptual(root, true, 0);
    let result = find_duplicates(&config).unwrap();

    assert!(result.groups.is_empty());
}
