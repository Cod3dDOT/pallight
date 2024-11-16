use eframe::Result;
use image::ImageError;
use lib_pxc::{decode, encode};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use thiserror::Error;

use crate::image_source::ImageSource;

#[derive(Error, Debug)]
pub enum ImageHandlingError {
    #[error("File dialog was canceled")]
    DialogCanceled,

    #[error("Invalid file path")]
    InvalidPath,

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Image processing error: {0}")]
    ImageError(#[from] ImageError),

    #[error("Custom format decode error")]
    DecodeError(#[from] DecodeError),

    #[error("Unsupported file extension")]
    UnsupportedExtension,
}

// Assuming you have a custom DecodeError for your format
#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Failed to decode custom image format")]
    DecodeFailed,
}

pub fn save_image(dimensions: (u32, u32), data: Vec<u8>) -> Result<(), ImageHandlingError> {
    let path = rfd::FileDialog::new()
        .save_file()
        .ok_or(ImageHandlingError::DialogCanceled)?;

    let path_str = path.to_str().ok_or(ImageHandlingError::InvalidPath)?;

    let mut file = File::create(path_str)?;

    let (width, height) = dimensions;
    let encoded_data = encode(width.try_into().unwrap(), height.try_into().unwrap(), &data)
        .map_err(|_| DecodeError::DecodeFailed)?;

    file.write_all(&encoded_data)?;
    println!("File saved successfully to {}", path_str);

    Ok(())
}

pub fn open_image() -> Result<Box<dyn ImageSource>, ImageHandlingError> {
    let path = rfd::FileDialog::new()
        .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "webp"])
        .add_filter(
            lib_pxc::constants::FORMAT_NAME,
            &[lib_pxc::constants::FILE_EXT],
        )
        .pick_file()
        .ok_or(ImageHandlingError::DialogCanceled)?;

    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or(ImageHandlingError::UnsupportedExtension)?;

    match ext {
        "pxc" => open_custom_image(&path),
        "png" | "jpg" | "jpeg" | "bmp" | "webp" => open_standard_image(&path),
        _ => Err(ImageHandlingError::UnsupportedExtension),
    }
}

fn open_custom_image(path: &PathBuf) -> Result<Box<dyn ImageSource>, ImageHandlingError> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let custom_image = decode(&buffer).map_err(|_| DecodeError::DecodeFailed)?;

    Ok(Box::new(custom_image))
}

fn open_standard_image(path: &PathBuf) -> Result<Box<dyn ImageSource>, ImageHandlingError> {
    Ok(Box::new(image::open(path)?))
}
