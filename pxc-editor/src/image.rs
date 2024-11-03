use eframe::egui::Color32;
use image::{DynamicImage, GenericImageView};
use lib_pxc::PXCImage;

use crate::chunk::{Chunk, CHUNK_SIZE};

// First, let's define a trait for image sources
pub trait ImageSource {
    /// Get the dimensions of the image (width, height)
    fn dims(&self) -> (u32, u32);

    /// Get a pixel at the specified coordinates
    /// Returns Color32 to maintain compatibility with the editor
    fn pixel(&self, x: u32, y: u32) -> Color32;

    /// Optional method to provide a more efficient way to load chunks directly
    /// Default implementation uses get_pixel for each pixel in the chunk
    fn load_chunk(&self, chunk_x: i32, chunk_y: i32) -> Option<Chunk> {
        let mut chunk = Chunk::new();
        let start_x = chunk_x * CHUNK_SIZE;
        let start_y = chunk_y * CHUNK_SIZE;
        let (width, height) = self.dims();

        let mut has_pixels = false;

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pixel_x = start_x + x;
                let pixel_y = start_y + y;

                if pixel_x >= 0
                    && pixel_y >= 0
                    && (pixel_x as u32) < width
                    && (pixel_y as u32) < height
                {
                    let color = self.pixel(pixel_x as u32, pixel_y as u32);
                    if color.a() > 0 {
                        has_pixels = true;
                    }
                    chunk.set_pixel(x, y, color);
                }
            }
        }

        println!("Loaded chunk ({}, {})", chunk_x, chunk_y);

        if has_pixels {
            Some(chunk)
        } else {
            None
        }
    }
}

// Implementation for the standard image crate's DynamicImage
impl ImageSource for DynamicImage {
    fn dims(&self) -> (u32, u32) {
        return DynamicImage::dimensions(&self);
    }

    fn pixel(&self, x: u32, y: u32) -> Color32 {
        let pixel = DynamicImage::get_pixel(&self, x, y);
        Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3])
    }
}

impl ImageSource for PXCImage {
    fn dims(&self) -> (u32, u32) {
        (self.width.into(), self.height.into())
    }

    fn pixel(&self, x: u32, y: u32) -> Color32 {
        let index = (y * self.width as u32 + x) as usize;
        if index >= self.rgba_data.len() {
            return Color32::BLACK;
        }
        println!(
            "{}: {} {} {} {}",
            index,
            self.rgba_data[index],
            self.rgba_data[index + 1],
            self.rgba_data[index + 2],
            self.rgba_data[index + 3]
        );
        Color32::from_rgba_unmultiplied(
            self.rgba_data[index],
            self.rgba_data[index + 1],
            self.rgba_data[index + 2],
            self.rgba_data[index + 3],
        )
    }
}
