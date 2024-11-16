mod common;

use common::{GRADIENT, RANDOM_RGB, REAL_IMAGE};
use lib_pxc::compression::{compress, decompress};

#[test]
fn test_comp_decomp_rgb() {
    // Compress
    let compressed = compress(&RANDOM_RGB).unwrap();

    // Verify compression results
    assert!(!compressed.data.is_empty());

    // Decompress
    let decompressed = decompress(compressed).unwrap();

    // Verify result
    assert_eq!(decompressed, &RANDOM_RGB);
}

#[test]
fn test_comp_decomp_repeating_color() {
    // Create test image with single color
    let rgba_data = vec![255, 0, 0, 255].repeat(16); // 4x4 red image

    // Compress
    let compressed = compress(&rgba_data).unwrap();

    // Single color should result in very good compression
    assert!(compressed.data.len() < rgba_data.len() / 4);

    // Decompress and verify
    let decompressed = decompress(compressed).unwrap();
    assert_eq!(decompressed, rgba_data);
}

#[test]
fn test_comp_decomp_gradients() {
    let compressed = compress(&GRADIENT).unwrap();

    // Decompress and verify
    let decompressed = decompress(compressed).unwrap();
    assert_eq!(decompressed, &GRADIENT);
}

#[test]
fn test_comp_decomp_real_image() {
    let compressed = compress(&REAL_IMAGE).unwrap();

    // Decompress and verify
    let decompressed = decompress(compressed).unwrap();
    assert_eq!(decompressed, &REAL_IMAGE);
}
