use std::collections::HashMap;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaletteCompressionError {
    #[error("Invalid pixel data length: expected multiple of 4 bytes, got {0}")]
    InvalidPixelDataLength(usize),
    #[error("Palette overflow: maximum 256 colors supported, attempted to add color #{0}")]
    PaletteOverflow(usize),
}

#[derive(Error, Debug)]
pub enum PaletteDecompressionError {
    #[error("Invalid palette index: {0} exceeds palette size of {1}")]
    InvalidPaletteIndex(usize, usize),
}

pub struct PaletteCompression {
    pub palette: Vec<[u8; 4]>, // Array of unique colors in RGBA format
    pub indices: Vec<u8>,      // Palette indices for each pixel
}

/// Compresses a raw RGBA pixel buffer to use a limited color palette.
///
/// # Parameters
/// - `pixels`: A slice of raw pixel data in RGBA format.
///
/// # Returns
/// A Result containing either a `PaletteCompression` struct with the color palette
/// and indices for each pixel, or a `PaletteError`.
///
/// # Errors
/// - Returns `PaletteError::InvalidPixelDataLength` if input length is not a multiple of 4
/// - Returns `PaletteError::PaletteOverflow` if more than 256 unique colors are found
pub fn palette_compression(pixels: &[u8]) -> Result<PaletteCompression, PaletteCompressionError> {
    // Validate input length
    if pixels.len() % 4 != 0 {
        return Err(PaletteCompressionError::InvalidPixelDataLength(
            pixels.len(),
        ));
    }

    let mut unique_colors = HashMap::new();
    let mut palette = Vec::new();
    let mut indices = Vec::with_capacity(pixels.len() / 4);

    // Iterate over each pixel (4 bytes: RGBA)
    for pixel in pixels.chunks(4) {
        let color = [pixel[0], pixel[1], pixel[2], pixel[3]];

        // Check if color is already in the palette
        if let Some(&index) = unique_colors.get(&color) {
            indices.push(index);
        } else {
            // Validate palette size before adding new color
            if palette.len() >= 256 {
                return Err(PaletteCompressionError::PaletteOverflow(palette.len() + 1));
            }

            let index = palette.len() as u8;
            palette.push(color);
            unique_colors.insert(color, index);
            indices.push(index);
        }
    }

    Ok(PaletteCompression { palette, indices })
}

/// Expands palette indices back into RGBA pixel data.
///
/// # Parameters
/// - `compression`: A PaletteCompression struct containing the palette and indices.
///
/// # Returns
/// A Result containing either a vector of raw pixel data in RGBA format or a `PaletteError`.
///
/// # Errors
/// - Returns `PaletteError::InvalidPaletteIndex` if any index exceeds the palette size
pub fn palette_decompression(
    compression: &PaletteCompression,
) -> Result<Vec<u8>, PaletteDecompressionError> {
    let mut decoded_pixels = Vec::with_capacity(compression.indices.len() * 4);

    for &index in &compression.indices {
        let palette_size = compression.palette.len();
        let index_usize = index as usize;

        // Validate index before accessing palette
        if index_usize >= palette_size {
            return Err(PaletteDecompressionError::InvalidPaletteIndex(
                index_usize,
                palette_size,
            ));
        }

        let color = compression.palette[index_usize];
        decoded_pixels.extend_from_slice(&color);
    }

    Ok(decoded_pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_rrgb() {
        let pixels = vec![
            255, 0, 0, 255, // Red
            255, 0, 0, 255, // Red
            0, 255, 0, 255, // Green
            0, 0, 255, 255, // Blue
        ];

        // Test compression
        let compressed = palette_compression(&pixels).unwrap();
        assert_eq!(compressed.palette.len(), 3); // Should have 3 unique colors
        assert_eq!(compressed.indices.len(), 4); // Should have 4 indices
        assert_eq!(compressed.indices[0], compressed.indices[1]); // First two pixels should have same index

        // Test decompression
        let decompressed = palette_decompression(&compressed).unwrap();
        assert_eq!(decompressed, pixels); // Should match original pixels
    }

    #[test]
    fn test_palette_invalid_pixel_data_length() {
        let pixels = vec![255, 0, 0]; // Invalid length (not multiple of 4)
        let result = palette_compression(&pixels);
        assert!(matches!(
            result,
            Err(PaletteCompressionError::InvalidPixelDataLength(3))
        ));
    }

    // #[test]
    // fn test_palette_overflow() {
    //     // Create 257 unique colors (should fail as max is 256)
    //     let mut pixels = Vec::with_capacity(257 * 4);
    //     for i in 0..258 {
    //         pixels.extend_from_slice(&[i as u8, 0, 0, 255]);
    //         println!("Pixel: {}", pixels[(i + 1) * 4 - 4]);
    //     }
    //     let result = palette_compression(&pixels);
    //     assert!(matches!(result, Err(PaletteError::PaletteOverflow(256))));
    // }

    #[test]
    fn test_palette_invalid_index() {
        let compressed = PaletteCompression {
            palette: vec![[255, 0, 0, 255]], // Palette with one color
            indices: vec![0, 1],             // Second index is invalid
        };
        let result = palette_decompression(&compressed);
        assert!(matches!(
            result,
            Err(PaletteDecompressionError::InvalidPaletteIndex(1, 1))
        ));
    }

    #[test]
    fn test_palette_empty_input() {
        let pixels: Vec<u8> = vec![];
        let compressed = palette_compression(&pixels).unwrap();
        assert!(compressed.palette.is_empty());
        assert!(compressed.indices.is_empty());

        let decompressed = palette_decompression(&compressed).unwrap();
        assert!(decompressed.is_empty());
    }

    #[test]
    fn test_palette_single_color() {
        let pixels = vec![100, 150, 200, 255];
        let compressed = palette_compression(&pixels).unwrap();
        assert_eq!(compressed.palette.len(), 1);
        assert_eq!(compressed.indices.len(), 1);
        assert_eq!(compressed.indices[0], 0);

        let decompressed = palette_decompression(&compressed).unwrap();
        assert_eq!(decompressed, pixels);
    }

    #[test]
    fn test_palette_gradients() {
        let mut data = Vec::new();
        for i in 0..256 {
            data.extend_from_slice(&[i as u8, i as u8, i as u8, 255]);
        }

        let compressed = palette_compression(&data).unwrap();
        let decompressed = palette_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }
}
