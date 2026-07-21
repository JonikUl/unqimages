use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, RgbImage};
use std::fs;
use std::path::PathBuf;

fn duplicate_pattern() -> RgbImage {
    let mut img = RgbImage::new(64, 64);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let v = (((x + y) * 2) % 256) as u8;
        *pixel = image::Rgb([v, v / 2, 255 - v]);
    }
    img
}

fn perceptual_original_pattern() -> RgbImage {
    let mut img = RgbImage::new(64, 64);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let v = ((x * 3 + y * 5) % 256) as u8;
        *pixel = image::Rgb([v, 128, 255 - v]);
    }
    img
}

fn unique_pattern() -> RgbImage {
    let mut img = RgbImage::new(64, 64);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let v = ((x * 7 + y * 13) % 256) as u8;
        *pixel = image::Rgb([255 - v, v, v / 3]);
    }
    img
}

fn different_pattern() -> RgbImage {
    let mut img = RgbImage::new(64, 64);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let v = ((x * 11 + y * 17) % 256) as u8;
        *pixel = image::Rgb([v / 2, 255 - v, v]);
    }
    img
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("test")
        .join("fixtures");

    fs::create_dir_all(&root).unwrap();

    let exact_dir = root.join("exact");
    let perceptual_dir = root.join("perceptual");
    let mixed_dir = root.join("mixed");
    let unsupported_dir = root.join("unsupported");

    fs::create_dir_all(&exact_dir).unwrap();
    fs::create_dir_all(&perceptual_dir).unwrap();
    fs::create_dir_all(&mixed_dir).unwrap();
    fs::create_dir_all(&unsupported_dir).unwrap();

    let duplicate = duplicate_pattern();
    let perceptual_original = perceptual_original_pattern();
    let unique = unique_pattern();
    let different = different_pattern();

    // Exact duplicates: byte-identical PNG files.
    duplicate.save(exact_dir.join("dupe-a.png")).unwrap();
    duplicate.save(exact_dir.join("dupe-b.png")).unwrap();
    unique.save(exact_dir.join("unique.png")).unwrap();

    // Perceptual duplicates: same image saved as PNG and compressed JPEG.
    perceptual_original
        .save(perceptual_dir.join("original.png"))
        .unwrap();
    let dynamic = DynamicImage::ImageRgb8(perceptual_original.clone());
    let mut jpeg_bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg_bytes, 80)
        .encode_image(&dynamic)
        .unwrap();
    fs::write(perceptual_dir.join("compressed.jpg"), &jpeg_bytes).unwrap();
    different
        .save(perceptual_dir.join("different.png"))
        .unwrap();

    // WebP copy of the duplicate for mixed-format exact duplicate detection.
    let duplicate_dynamic = DynamicImage::ImageRgb8(duplicate.clone());
    let mut webp_bytes = Vec::new();
    duplicate_dynamic
        .write_to(
            &mut std::io::Cursor::new(&mut webp_bytes),
            ImageFormat::WebP,
        )
        .unwrap();
    fs::write(mixed_dir.join("dupe.webp"), &webp_bytes).unwrap();

    // Unsupported file that must be ignored by discovery.
    fs::write(unsupported_dir.join("file.txt"), "not an image").unwrap();

    println!("Fixtures generated in {}", root.display());
}
