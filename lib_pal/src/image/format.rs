pub const MAGIC_HEADER: [u8; 4] = *b"PXC1";

#[derive(Debug)]
pub struct Image {
    pub magic: [u8; 4],
    pub width: u16,
    pub height: u16,
    pub palette_size: u8,
    pub palette: Vec<[u8; 4]>,
    pub rgba_data: Vec<u8>,
}

impl Image {
    pub const MAGIC_SIZE: usize = 4;
    pub const WIDTH_HEIGHT_SIZE: usize = std::mem::size_of::<u16>();
    pub const PALETTE_SIZE_SIZE: usize = std::mem::size_of::<u8>();

    pub fn new(
        width: u16,
        height: u16,
        palette_size: u8,
        palette: Vec<[u8; 4]>,
        rgba_data: Vec<u8>,
    ) -> Self {
        Self {
            magic: MAGIC_HEADER,
            width,
            height,
            palette_size,
            palette,
            rgba_data,
        }
    }
}
