use eframe::egui::Color32;
use image::{DynamicImage, GenericImageView};
use lib_pxc::PXCImage;

use crate::chunk::{Chunk, CHUNK_SIZE};

// First, let's define a trait for image sources
pub trait ImageSource {
    /// Get the dimensions of the image (width, height)
    fn dimensions(&self) -> (u32, u32);

    /// Get a pixel at the specified coordinates
    /// Returns Color32 to maintain compatibility with the editor
    fn get_pixel(&self, x: u32, y: u32) -> Color32;

    /// Optional method to provide a more efficient way to load chunks directly
    /// Default implementation uses get_pixel for each pixel in the chunk
    fn load_chunk(&self, chunk_x: i32, chunk_y: i32) -> Option<Chunk>;
}

// Implementation for the standard image crate's DynamicImage
impl ImageSource for DynamicImage {
    fn dimensions(&self) -> (u32, u32) {
        return GenericImageView::dimensions(self);
    }

    fn get_pixel(&self, x: u32, y: u32) -> Color32 {
        let pixel = GenericImageView::get_pixel(self, x, y);
        Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3])
    }

    fn load_chunk(&self, chunk_x: i32, chunk_y: i32) -> Option<Chunk> {
        let (img_width, img_height) = ImageSource::dimensions(self);
        let start_x = (chunk_x * CHUNK_SIZE as i32) as u32;
        let start_y = (chunk_y * CHUNK_SIZE as i32) as u32;

        if start_x >= img_width || start_y >= img_height {
            return None;
        }

        let mut chunk = Chunk::new();
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pixel_x = start_x + x as u32;
                let pixel_y = start_y + y as u32;

                if pixel_x < img_width && pixel_y < img_height {
                    let pixel = ImageSource::get_pixel(self, pixel_x, pixel_y);
                    let rgba =
                        Color32::from_rgba_premultiplied(pixel[0], pixel[1], pixel[2], pixel[3]);
                    chunk.set_pixel(x, y, rgba).ok();
                }
            }
        }

        Some(chunk)
    }
}

impl ImageSource for PXCImage {
    fn dimensions(&self) -> (u32, u32) {
        (self.width.into(), self.height.into())
    }

    fn get_pixel(&self, x: u32, y: u32) -> Color32 {
        if x >= self.width.into() || y >= self.height.into() {
            return Color32::BLACK;
        }
        let x = x as u16;
        let y = y as u16;
        let index = (y * self.width + x) as usize;

        println!(
            "Pixel ({}, {}): rgba = ({}, {}, {}, {})",
            x,
            y,
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

    fn load_chunk(&self, chunk_x: i32, chunk_y: i32) -> Option<Chunk> {
        let start_x = (chunk_x * CHUNK_SIZE as i32) as u32;
        let start_y = (chunk_y * CHUNK_SIZE as i32) as u32;

        if start_x >= self.width.into() || start_y >= self.height.into() {
            return None;
        }

        println!("Loading image ({:?})", self.rgba_data);

        let mut chunk = Chunk::new();
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pixel_x = start_x + x as u32;
                let pixel_y = start_y + y as u32;

                if pixel_x < self.width.into() && pixel_y < self.height.into() {
                    let color = self.get_pixel(pixel_x, pixel_y);
                    // println!("Pixel ({}, {}): {:?}", pixel_x, pixel_y, color);
                    chunk.set_pixel(x, y, color).ok();
                }
            }
        }

        Some(chunk)
    }
}
