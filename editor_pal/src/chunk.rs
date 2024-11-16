use eframe::egui::Color32;
use thiserror::Error;

pub const CHUNK_SIZE: u8 = 64;

#[derive(Error, Debug)]
pub enum ChunkError {
    #[error("coordinates ({x}, {y}) are out of bounds for chunk size {chunk_size}")]
    OutOfBounds { x: u8, y: u8, chunk_size: u8 },
}

pub struct Chunk {
    pub pixels: Box<[Color32; CHUNK_SIZE as usize * CHUNK_SIZE as usize]>,
    pub is_empty: bool,
}

impl Chunk {
    pub fn new() -> Self {
        let default_pixels =
            Box::new([Color32::TRANSPARENT; CHUNK_SIZE as usize * CHUNK_SIZE as usize]);
        Self {
            pixels: default_pixels,
            is_empty: true,
        }
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, color: Color32) -> Result<usize, ChunkError> {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE {
            return Err(ChunkError::OutOfBounds {
                x,
                y,
                chunk_size: CHUNK_SIZE,
            });
        }

        let index = y as usize * CHUNK_SIZE as usize + x as usize;
        self.pixels[index] = color;
        if color != Color32::TRANSPARENT {
            self.is_empty = false;
        } else {
            self.is_empty = self
                .pixels
                .iter()
                .all(|&pixel| pixel == Color32::TRANSPARENT);
        }

        Ok(index)
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> Result<Color32, ChunkError> {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE {
            return Err(ChunkError::OutOfBounds {
                x,
                y,
                chunk_size: CHUNK_SIZE,
            });
        }

        let index = (y * CHUNK_SIZE + x) as usize;
        Ok(self.pixels[index])
    }
}
