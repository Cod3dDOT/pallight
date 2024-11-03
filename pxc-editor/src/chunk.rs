use std::collections::HashMap;

use eframe::egui::Color32;

use crate::image::ImageSource;

pub const CHUNK_SIZE: i32 = 32;

// A chunk of pixels (32x32)
pub struct Chunk {
    pub pixels: Box<[Color32; CHUNK_SIZE as usize * CHUNK_SIZE as usize]>,
    pub is_empty: bool,
}

impl Chunk {
    pub fn new() -> Self {
        let default_pixels = Box::new([Color32::TRANSPARENT; (CHUNK_SIZE * CHUNK_SIZE) as usize]);
        Self {
            pixels: default_pixels,
            is_empty: true,
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color32) {
        let index = (y * CHUNK_SIZE + x) as usize;
        self.pixels[index] = color;
        self.is_empty = false;
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Color32 {
        let index = (y * CHUNK_SIZE + x) as usize;
        self.pixels[index]
    }
}

#[derive(Default)]
pub struct ChunkedImage {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub bounds: Option<((i32, i32), (i32, i32))>, // ((min_x, min_y), (max_x, max_y))
}

impl ChunkedImage {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            bounds: None,
        }
    }

    pub fn load_from_source(&mut self, source: &impl ImageSource) {
        self.clear();

        let (width, height) = source.dims();
        let chunks_width = (width as f32 / CHUNK_SIZE as f32).ceil() as i32;
        let chunks_height = (height as f32 / CHUNK_SIZE as f32).ceil() as i32;

        println!("Chunks: {}x{}", chunks_width, chunks_height);

        // Load chunks efficiently
        for chunk_y in 0..chunks_height {
            for chunk_x in 0..chunks_width {
                if let Some(chunk) = source.load_chunk(chunk_x, chunk_y) {
                    self.chunks.insert((chunk_x, chunk_y), chunk);
                }
            }
        }

        // Update bounds
        if width > 0 && height > 0 {
            self.bounds = Some(((0, 0), (width as i32 - 1, height as i32 - 1)));
        }

        if let Some(((min_x, min_y), (max_x, max_y))) = &self.bounds {
            println!(
                "Bounds: min_x: {}, min_y: {}, max_x: {}, max_y: {}",
                min_x, min_y, max_x, max_y
            );
        } else {
            println!("Bounds: None");
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color32) {
        let chunk_x = x.div_euclid(CHUNK_SIZE);
        let chunk_y = y.div_euclid(CHUNK_SIZE);
        let local_x = x.rem_euclid(CHUNK_SIZE);
        let local_y = y.rem_euclid(CHUNK_SIZE);

        let chunk = self
            .chunks
            .entry((chunk_x, chunk_y))
            .or_insert_with(Chunk::new);
        chunk.set_pixel(local_x, local_y, color);

        // Update bounds
        match self.bounds {
            None => self.bounds = Some(((x, y), (x, y))),
            Some(((min_x, min_y), (max_x, max_y))) => {
                self.bounds = Some(((min_x.min(x), min_y.min(y)), (max_x.max(x), max_y.max(y))));
            }
        }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Color32 {
        let chunk_x = x.div_euclid(CHUNK_SIZE);
        let chunk_y = y.div_euclid(CHUNK_SIZE);
        let local_x = x.rem_euclid(CHUNK_SIZE);
        let local_y = y.rem_euclid(CHUNK_SIZE);

        self.chunks
            .get(&(chunk_x, chunk_y))
            .map_or(Color32::TRANSPARENT, |chunk| {
                chunk.get_pixel(local_x, local_y)
            })
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.bounds = None;
    }
}
