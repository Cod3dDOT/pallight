use log::{debug, error, info};
use thiserror::Error;

use super::format::MAGIC_HEADER;
use crate::compression::{compress, CompressionError};

#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("Failed to compress image data")]
    CompressionFailed(#[from] CompressionError),
    #[error("Palette size exceeds 256 colors")]
    PaletteTooLarge,
}

pub fn encode(width: u16, height: u16, rgba_data: &[u8]) -> Result<Vec<u8>, EncodingError> {
    info!("Starting encoding");

    let mut encoded_data: Vec<u8> = Vec::new();

    // Step 1: Write header
    encoded_data.extend_from_slice(&MAGIC_HEADER); // Magic Number
    encoded_data.extend_from_slice(&width.to_be_bytes()); // Width
    encoded_data.extend_from_slice(&height.to_be_bytes()); // Height
    debug!(
        "Header written:\nMagic: {:?}\nWidth: {}\nHeight: {}",
        MAGIC_HEADER, width, height
    );

    // Step 2: Compress the image data
    let compressed_data = compress(rgba_data)?;
    debug!(
        "Image data compressed successfully with palette size: {}",
        compressed_data.palette.len()
    );

    // Check that the palette size does not exceed 256 colors
    if compressed_data.palette.len() > 256 {
        error!(
            "Palette size {} exceeds the maximum allowed limit of 256 colors",
            compressed_data.palette.len()
        );
        return Err(EncodingError::PaletteTooLarge);
    }
    encoded_data.push(compressed_data.palette.len() as u8);
    debug!("Palette size added to encoded data");

    // Add palette data (each color is [u8; 4])
    for color in &compressed_data.palette {
        encoded_data.extend_from_slice(color);
    }
    debug!(
        "Palette data written with {} colors",
        compressed_data.palette.len()
    );

    // Add LZW-compressed indices directly
    encoded_data.extend_from_slice(&compressed_data.data);
    debug!("LZW-compressed indices added to encoded data");

    info!("Encoding process completed successfully");
    Ok(encoded_data)
}
