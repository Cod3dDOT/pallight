pub mod huffman;
pub mod image;
pub mod lzw;
pub mod palette;
pub mod rle_delta;

use log::{debug, error, info};
use lzw::{LzwCompressionError, LzwDecompressionError};
use palette::{PaletteCompressionError, PaletteDecompressionError};
use rle_delta::{RleCompressionError, RleDecompressionError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Palette compression failed")]
    PaletteCompressionFailed(#[from] PaletteCompressionError),
    #[error("RLE Delta compression failed")]
    RleDeltaCompressionFailed(#[from] RleCompressionError),
    #[error("LZW compression failed")]
    LzwCompressionFailed(#[from] LzwCompressionError),
}

#[derive(Error, Debug)]
pub enum DecompressionError {
    #[error("Palette decompression failed")]
    PaletteDecompressionFailed(#[from] PaletteDecompressionError),
    #[error("RLE Delta decompression failed")]
    RleDeltaDecompressionFailed(#[from] RleDecompressionError),
    #[error("LZW decompression failed")]
    LzwDecompressionFailed(#[from] LzwDecompressionError),
}

pub struct CompressionResult {
    pub palette: Vec<[u8; 4]>,
    pub data: Vec<u8>,
}

pub fn compress(data: &[u8]) -> Result<CompressionResult, CompressionError> {
    info!("Starting compression");

    debug!("Input data length: {}", data.len());
    debug!("Input data: {:?}\n\n", data);

    // Step 1: Palette Compression
    let palette_compressed = palette::palette_compression(data)?;
    debug!(
        "Palette compressed: {} unique colors, {} bytes",
        palette_compressed.palette.len(),
        palette_compressed.indices.len()
    );
    debug!("Palette: {:?}", palette_compressed.palette);
    debug!("Palette indices: {:?}\n\n", palette_compressed.indices);

    // Step 2: RLE Delta Encoding
    let rle_delta_encoded = rle_delta::rle_delta_compression(&palette_compressed.indices)?;
    debug!("RLE Delta encoding: {} bytes", rle_delta_encoded.len());
    debug!("RLE Delta encoded data: {:?}\n\n", rle_delta_encoded);

    // Step 3: LZW Compression
    let lzw_compressed = lzw::lzw_compression(&rle_delta_encoded)?;
    debug!("LZW compression: {} bytes", lzw_compressed.len());
    debug!("LZW compressed data: {:?}\n\n", lzw_compressed);

    info!(
        "Compression completed successfully: {}%",
        ((data.len() as f32 - lzw_compressed.len() as f32) / data.len() as f32) * 100.0
    );

    Ok(CompressionResult {
        palette: palette_compressed.palette,
        data: lzw_compressed,
    })
}

pub fn decompress(data: CompressionResult) -> Result<Vec<u8>, DecompressionError> {
    info!("Starting decompression");

    debug!("Input data length: {}", data.data.len());
    debug!("Input data: {:?}\n\n", data.data);

    // Step 1: LZW Decompression
    let lzw_decompressed = lzw::lzw_decompression(&data.data)?;
    debug!("LZW decompression: {} bytes", lzw_decompressed.len());
    debug!("LZW decompressed data: {:?}\n\n", lzw_decompressed);

    // Step 2: RLE and Delta Decoding
    let rle_delta_decoded = rle_delta::rle_delta_decompression(&lzw_decompressed)?;
    debug!("RLE Delta decoding: {} bytes", rle_delta_decoded.len());
    debug!("RLE Delta decoded data: {:?}\n\n", rle_delta_decoded);

    // Step 3: Palette Expansion to RGBA
    let expanded_pixels = palette::palette_decompression(&palette::PaletteCompression {
        palette: data.palette,
        indices: rle_delta_decoded,
    })?;
    debug!("Palette expansion: {} bytes", expanded_pixels.len());
    debug!("Palette expanded data: {:?}\n\n", expanded_pixels);

    info!("Decompression completed successfully");

    Ok(expanded_pixels)
}
