use std::collections::HashMap;

use crate::{
    chunk::{Chunk, ChunkError, CHUNK_SIZE},
    image_source::ImageSource,
};
use eframe::egui::{self, Color32, TextureHandle};

pub const CELL_SIZE: i32 = 20; // in pixels

#[derive(Default)]
pub struct Canvas {
    cached_visible_chunks: HashMap<(u32, u32), TextureHandle>,
    chunks: HashMap<(u32, u32), Chunk>,
    width: u32,
    height: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            cached_visible_chunks: HashMap::new(),
            chunks: HashMap::new(),
            width,
            height,
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_chunk_coords(x: u32, y: u32) -> (u32, u32) {
        let chunk_x = x / CHUNK_SIZE as u32;
        let chunk_y = y / CHUNK_SIZE as u32;
        (chunk_x, chunk_y)
    }

    pub fn get_local_coords(x: u32, y: u32) -> (u8, u8) {
        let local_x = (x % CHUNK_SIZE as u32) as u8;
        let local_y = (y % CHUNK_SIZE as u32) as u8;
        (local_x, local_y)
    }

    pub fn get_absolute_coords(chunk_x: u32, chunk_y: u32, local_x: u8, local_y: u8) -> (u32, u32) {
        let x = chunk_x * CHUNK_SIZE as u32 + local_x as u32;
        let y = chunk_y * CHUNK_SIZE as u32 + local_y as u32;
        (x, y)
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color32) -> Result<(), ChunkError> {
        if x >= self.width || y >= self.height {
            return Err(ChunkError::OutOfBounds {
                x: x as u8,
                y: y as u8,
                chunk_size: CHUNK_SIZE,
            });
        }

        let chunk_coords = Self::get_chunk_coords(x, y);
        let local_coords = Self::get_local_coords(x, y);

        println!(
            "Setting pixel at ({}, {}), chunk coords: ({}, {}), local coords: ({}, {})",
            x, y, chunk_coords.0, chunk_coords.1, local_coords.0, local_coords.1
        );

        let chunk = self.chunks.entry(chunk_coords).or_insert_with(Chunk::new);
        self.cached_visible_chunks.remove(&chunk_coords);

        chunk.set_pixel(local_coords.0, local_coords.1, color)?;

        Ok(())
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Result<Color32, ChunkError> {
        if x >= self.width || y >= self.height {
            return Err(ChunkError::OutOfBounds {
                x: x as u8,
                y: y as u8,
                chunk_size: CHUNK_SIZE,
            });
        }

        let chunk_coords = Self::get_chunk_coords(x, y);
        let local_coords = Self::get_local_coords(x, y);

        self.chunks
            .get(&chunk_coords)
            .map_or(Ok(Color32::TRANSPARENT), |chunk| {
                chunk.get_pixel(local_coords.0, local_coords.1)
            })
    }

    pub fn get_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for chunk in self.chunks.values() {
            data.extend_from_slice(
                &chunk
                    .pixels
                    .iter()
                    .flat_map(|p| vec![p.r(), p.g(), p.b(), p.a()])
                    .collect::<Vec<u8>>(),
            );
        }
        data
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.cached_visible_chunks.clear();
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;

        // Remove chunks outside the new canvas bounds
        self.chunks.retain(|&(chunk_x, chunk_y), _| {
            let chunk_min_x = chunk_x * CHUNK_SIZE as u32;
            let chunk_min_y = chunk_y * CHUNK_SIZE as u32;
            chunk_min_x < new_width && chunk_min_y < new_height
        });
    }

    pub fn load_image(&mut self, image: &impl ImageSource) {
        self.clear();

        let (width, height) = image.dimensions();

        self.resize(width, height);

        let chunks_width = (width as f32 / CHUNK_SIZE as f32).ceil() as u32;
        let chunks_height = (height as f32 / CHUNK_SIZE as f32).ceil() as u32;

        for chunk_y in 0..chunks_height {
            for chunk_x in 0..chunks_width {
                if let Some(chunk) = image.load_chunk(chunk_x as i32, chunk_y as i32) {
                    self.chunks.insert((chunk_x, chunk_y), chunk);
                }
            }
        }
    }

    pub fn update_cache(
        &mut self,
        visible_chunks: &Vec<(u32, u32)>,
        ctx: &egui::Context,
    ) -> &HashMap<(u32, u32), TextureHandle> {
        // Retain only the visible chunks in the cache
        self.cached_visible_chunks
            .retain(|chunk_pos, _| visible_chunks.contains(chunk_pos));

        for &chunk_pos in visible_chunks {
            // Check if the chunk is not already cached
            if !self.cached_visible_chunks.contains_key(&chunk_pos) {
                // Only render the chunk if it exists
                if let Some(chunk) = self.chunks.get(&chunk_pos) {
                    if let Some(texture) = Self::render_chunk(chunk, chunk_pos, ctx) {
                        self.cached_visible_chunks.insert(chunk_pos, texture);
                    }
                }
            }
        }

        &self.cached_visible_chunks
    }

    fn render_chunk(
        chunk: &Chunk,
        chunk_pos: (u32, u32),
        ctx: &egui::Context,
    ) -> Option<egui::TextureHandle> {
        if chunk.is_empty {
            return None;
        }

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [CHUNK_SIZE as usize, CHUNK_SIZE as usize],
            &chunk
                .pixels
                .iter()
                .flat_map(|p| vec![p.r(), p.g(), p.b(), p.a()])
                .collect::<Vec<u8>>(),
        );

        Some(ctx.load_texture(
            format!("chunk_{}_{}", chunk_pos.0, chunk_pos.1),
            color_image,
            egui::TextureOptions::NEAREST,
        ))
    }
}
