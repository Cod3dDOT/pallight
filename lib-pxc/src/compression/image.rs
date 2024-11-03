use std::fs::File;
use std::io::{self, Read, Write};

pub const EXTENSION: &str = "pxc";

pub struct PXCImage {
    pub version: u8,
    pub width: u8,
    pub height: u8,
    pub palette: Vec<[u8; 4]>,
    pub data: Vec<u16>,
}

impl PXCImage {
    pub fn new(version: u8, width: u8, height: u8, palette: Vec<[u8; 4]>, data: Vec<u16>) -> Self {
        Self {
            version,
            width,
            height,
            palette,
            data,
        }
    }

    /// Saves the compressed file in a custom binary format
    pub fn save(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;

        // Write metadata
        file.write_all(&[self.version, self.width, self.height])?;

        // Write palette size and colors
        file.write_all(&(self.palette.len() as u8).to_le_bytes())?;
        for color in &self.palette {
            file.write_all(color)?;
        }

        // Write compressed data size and data
        file.write_all(&(self.data.len() as u16).to_le_bytes())?;
        for &code in &self.data {
            file.write_all(&code.to_le_bytes())?;
        }

        Ok(())
    }

    /// Loads a compressed file from a custom binary format
    pub fn load(filename: &str) -> io::Result<Self> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Read metadata
        let version = buffer[0];
        let width = buffer[1];
        let height = buffer[2];

        // Read palette
        let palette_size = buffer[3] as usize;
        let mut palette = Vec::new();
        let mut offset = 4;
        for _ in 0..palette_size {
            palette.push([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            offset += 4;
        }

        // Read compressed data
        let data_size = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]) as usize;
        offset += 2;
        let mut data = Vec::new();
        for _ in 0..data_size {
            let code = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
            data.push(code);
            offset += 2;
        }

        Ok(Self {
            version,
            width,
            height,
            palette,
            data,
        })
    }
}
