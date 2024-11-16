use super::format::{Image, MAGIC_HEADER};
use crate::compression::{decompress, CompressionResult, DecompressionError};
use log::{debug, error, info};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Invalid format or header")]
    InvalidHeader,
    #[error("Unexpected end of data while reading palette color #{0}")]
    UnexpectedEofPaletteColor(usize),
    #[error("Insufficient data for palette size")]
    InsufficientDataForPaletteSize,
    #[error("Failed to parse image dimensions")]
    DimensionParsingFailed,

    #[error("Decompression failed")]
    DecompressionFailed(#[from] DecompressionError),
}

pub fn decode(encoded_data: &[u8]) -> Result<Image, DecodeError> {
    let mut cursor = 0;

    // Check the header and magic number
    if encoded_data.len() < Image::MAGIC_SIZE || !encoded_data.starts_with(&MAGIC_HEADER) {
        error!("Invalid format or missing magic number in header");
        return Err(DecodeError::InvalidHeader);
    }
    debug!("Magic number validated successfully");
    cursor += Image::MAGIC_SIZE;

    // Read width and height
    let width = u16::from_be_bytes(
        encoded_data[cursor..cursor + Image::WIDTH_HEIGHT_SIZE]
            .try_into()
            .map_err(|_| {
                error!("Failed to parse width");
                DecodeError::DimensionParsingFailed
            })?,
    );
    cursor += Image::WIDTH_HEIGHT_SIZE;
    let height = u16::from_be_bytes(
        encoded_data[cursor..cursor + Image::WIDTH_HEIGHT_SIZE]
            .try_into()
            .map_err(|_| {
                error!("Failed to parse height");
                DecodeError::DimensionParsingFailed
            })?,
    );
    cursor += Image::WIDTH_HEIGHT_SIZE;
    debug!("Image dimensions read: width={} height={}", width, height);

    // Read palette size
    if cursor >= encoded_data.len() {
        error!("Insufficient data for palette size");
        return Err(DecodeError::InsufficientDataForPaletteSize);
    }
    let palette_size = encoded_data[cursor] as usize;
    cursor += Image::PALETTE_SIZE_SIZE;
    debug!("Palette size: {}", palette_size);

    // Read palette
    let mut palette = Vec::with_capacity(palette_size);
    for i in 0..palette_size {
        if cursor + 4 > encoded_data.len() {
            error!("Unexpected end of data while reading palette color #{}", i);
            return Err(DecodeError::UnexpectedEofPaletteColor(i));
        }
        let color = [
            encoded_data[cursor],
            encoded_data[cursor + 1],
            encoded_data[cursor + 2],
            encoded_data[cursor + 3],
        ];
        palette.push(color);
        cursor += 4;
        debug!("Read palette color #{}: {:?}", i, color);
    }

    // The remaining data is compressed image data
    let compressed_data = &encoded_data[cursor..];
    debug!("Compressed data length: {}", compressed_data.len());

    // Perform decompression
    let rgba_data = decompress(CompressionResult {
        palette: palette.clone(),
        data: compressed_data.to_vec(),
    })?;
    info!("Decompression successful");

    // Return the decoded image
    Ok(Image::new(
        width,
        height,
        palette_size as u8,
        palette,
        rgba_data,
    ))
}
