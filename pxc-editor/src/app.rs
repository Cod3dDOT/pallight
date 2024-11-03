use crate::chunk::Chunk;
use eframe::egui::{self, Align, Layout};
use eframe::Frame;
use egui::{Color32, Pos2, Rect, Vec2};
use lib_pxc::{decode, encode};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

use crate::chunk::{ChunkedImage, CHUNK_SIZE};
use crate::image::ImageSource;

const DEFAULT_COLORS: &[Color32] = &[Color32::BLACK, Color32::WHITE, Color32::BLUE];

#[derive(Default)]
pub struct PixelEditor {
    image: ChunkedImage,
    pan_offset: Vec2,
    zoom: f32,
    current_color: usize,
    last_mouse_pos: Option<Pos2>,
    colors: Vec<Color32>,
    cached_visible_chunks: HashMap<(i32, i32), egui::TextureHandle>,
    draw_grid: bool,
}

impl PixelEditor {
    pub fn new() -> Self {
        Self {
            image: ChunkedImage::new(),
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            current_color: 0,
            last_mouse_pos: None,
            colors: DEFAULT_COLORS.to_vec(),
            cached_visible_chunks: HashMap::new(),
            draw_grid: true,
        }
    }

    fn calculate_zoom_to_fit(&self, view_size: Vec2) -> f32 {
        if let Some(((min_x, min_y), (max_x, max_y))) = self.image.bounds {
            let image_width = (max_x - min_x + 1) as f32;
            let image_height = (max_y - min_y + 1) as f32;

            // Calculate zoom levels that would fit the image in each dimension
            let zoom_x = view_size.x / (image_width * 20.0);
            let zoom_y = view_size.y / (image_height * 20.0);

            // Use the smaller zoom level to ensure the entire image fits
            let zoom = zoom_x.min(zoom_y);

            // Clamp zoom to reasonable bounds
            zoom.clamp(0.1, 10.0)
        } else {
            1.0 // Default zoom if there's no image
        }
    }

    fn center_view(&mut self, view_size: Vec2) {
        if let Some(((min_x, min_y), (max_x, max_y))) = self.image.bounds {
            // Calculate the width and height of the image
            let image_width = (max_x - min_x + 1) as f32;
            let image_height = (max_y - min_y + 1) as f32;

            // Calculate the center of the image in grid coordinates
            let image_center_x = image_width / 2.0;
            let image_center_y = image_height / 2.0;

            // Calculate the center of the view
            let view_center_x = view_size.x / 2.0;
            let view_center_y = view_size.y / 2.0;

            // Scale image center based on the current zoom level and cell size (e.g., 20.0)
            let scaled_image_center_x = image_center_x * self.zoom * 20.0;
            let scaled_image_center_y = image_center_y * self.zoom * 20.0;

            // Calculate the offset to center the image in the view
            self.pan_offset = Vec2::new(
                view_center_x - scaled_image_center_x,
                view_center_y - scaled_image_center_y,
            );
        }
    }

    fn handle_image_load<T: ImageSource>(&mut self, source: &T, view_size: Vec2) {
        self.cached_visible_chunks.clear();
        self.image.load_from_source(source);

        // Center the view on the loaded image and adjust zoom
        self.center_view(view_size);
        // self.zoom = self.calculate_zoom_to_fit(view_size);
    }

    fn screen_to_grid(&self, pos: Pos2) -> (i32, i32) {
        let adjusted_pos = pos - self.pan_offset;
        let cell_size = self.zoom * 20.0;
        let x = (adjusted_pos.x / cell_size).floor() as i32;
        let y = (adjusted_pos.y / cell_size).floor() as i32;
        (x, y)
    }

    fn grid_to_screen(&self, grid_pos: (i32, i32)) -> Rect {
        let cell_size = self.zoom * 20.0;
        let x = grid_pos.0 as f32 * cell_size + self.pan_offset.x;
        let y = grid_pos.1 as f32 * cell_size + self.pan_offset.y;
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(cell_size, cell_size))
    }

    fn get_visible_chunks(&self, rect: Rect) -> Vec<(i32, i32)> {
        let top_left = self.screen_to_grid(rect.min);
        let bottom_right = self.screen_to_grid(rect.max);

        let chunk_start_x = top_left.0.div_euclid(CHUNK_SIZE);
        let chunk_start_y = top_left.1.div_euclid(CHUNK_SIZE);
        let chunk_end_x = bottom_right.0.div_euclid(CHUNK_SIZE);
        let chunk_end_y = bottom_right.1.div_euclid(CHUNK_SIZE);

        let mut chunks = Vec::new();
        for chunk_y in chunk_start_y..=chunk_end_y {
            for chunk_x in chunk_start_x..=chunk_end_x {
                chunks.push((chunk_x, chunk_y));
            }
        }
        chunks
    }

    fn render_chunk(
        chunk: &Chunk,
        chunk_pos: (i32, i32),
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
                .flat_map(|c| vec![c.r(), c.g(), c.b(), c.a()])
                .collect::<Vec<_>>(),
        );

        Some(ctx.load_texture(
            format!("chunk_{}_{}", chunk_pos.0, chunk_pos.1),
            color_image,
            egui::TextureOptions::NEAREST,
        ))
    }

    fn draw_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_toolbar(ui, ui.available_size());
            ui.separator();

            let response = ui.allocate_response(ui.available_size(), egui::Sense::drag());
            let input = ui.input(|i| i.clone());

            // Handle zooming
            if input.modifiers.ctrl {
                let scroll_delta = input.raw_scroll_delta.y;
                if scroll_delta != 0.0 {
                    let old_zoom = self.zoom;
                    let zoom_delta = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                    self.zoom *= zoom_delta;
                    self.zoom = self.zoom.clamp(0.1, 10.0);

                    // Adjust pan offset to zoom towards cursor
                    if let Some(mouse_pos) = input.pointer.hover_pos() {
                        let zoom_center = mouse_pos - self.pan_offset;
                        let zoom_ratio = self.zoom / old_zoom;
                        self.pan_offset = mouse_pos - zoom_center * zoom_ratio;
                    }
                }
            }

            // Handle panning
            if input.pointer.middle_down() {
                if let Some(mouse_pos) = input.pointer.hover_pos() {
                    if let Some(last_pos) = self.last_mouse_pos {
                        let delta = mouse_pos - last_pos;
                        self.pan_offset += delta;
                    }
                    self.last_mouse_pos = Some(mouse_pos);
                }
            } else {
                self.last_mouse_pos = None;
            }

            let painter = ui.painter();
            let rect = response.rect;

            // Get visible chunks
            let visible_chunks = self.get_visible_chunks(rect);

            // Update texture cache
            let mut new_cache = HashMap::new();

            // First, move still-visible textures to the new cache
            for chunk_pos in &visible_chunks {
                if let Some(texture) = self.cached_visible_chunks.remove(chunk_pos) {
                    new_cache.insert(*chunk_pos, texture);
                }
            }

            // Then, render any new chunks that aren't cached
            for chunk_pos in &visible_chunks {
                if !new_cache.contains_key(chunk_pos) {
                    if let Some(chunk) = self.image.chunks.get(chunk_pos) {
                        if let Some(texture) = Self::render_chunk(chunk, *chunk_pos, ctx) {
                            new_cache.insert(*chunk_pos, texture);
                        }
                    }
                }
            }

            // Replace the old cache with the new one
            self.cached_visible_chunks = new_cache;

            // Draw the chunks
            for &chunk_pos in &visible_chunks {
                if let Some(texture) = self.cached_visible_chunks.get(&chunk_pos) {
                    let chunk_rect = Rect::from_min_size(
                        Pos2::new(
                            (chunk_pos.0 * CHUNK_SIZE) as f32 * self.zoom * 20.0
                                + self.pan_offset.x,
                            (chunk_pos.1 * CHUNK_SIZE) as f32 * self.zoom * 20.0
                                + self.pan_offset.y,
                        ),
                        Vec2::new(
                            CHUNK_SIZE as f32 * self.zoom * 20.0,
                            CHUNK_SIZE as f32 * self.zoom * 20.0,
                        ),
                    );

                    if chunk_rect.intersects(rect) {
                        painter.image(
                            texture.id(),
                            chunk_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                }
            }

            // Draw grid if enabled and zoom level is high enough
            if self.draw_grid && self.zoom >= 0.5 {
                let top_left = self.screen_to_grid(rect.min);
                let bottom_right = self.screen_to_grid(rect.max);

                for x in top_left.0..=bottom_right.0 {
                    for y in top_left.1..=bottom_right.1 {
                        let cell_rect = self.grid_to_screen((x, y));
                        painter.rect_stroke(cell_rect, 0.0, egui::Stroke::new(0.2, Color32::GRAY));
                    }
                }
            }

            // Handle drawing
            if response.clicked() || (response.dragged() && input.pointer.primary_down()) {
                if let Some(pos) = input.pointer.hover_pos() {
                    let grid_pos = self.screen_to_grid(pos);
                    self.image
                        .set_pixel(grid_pos.0, grid_pos.1, self.colors[self.current_color]);

                    // When a pixel is modified, remove the affected chunk's texture from cache
                    let chunk_pos = (
                        grid_pos.0.div_euclid(CHUNK_SIZE),
                        grid_pos.1.div_euclid(CHUNK_SIZE),
                    );
                    self.cached_visible_chunks.remove(&chunk_pos);
                }
            }
        });
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui, view_size: Vec2) {
        ui.horizontal(|ui| {
            // Load image button
            if ui.button("📂 Load Image").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "webp"])
                    .add_filter("PXC", &["pxc"])
                    .pick_file()
                {
                    if let Some(ext) = path.extension() {
                        if ext == "pxc" {
                            if let Ok(mut file) = File::open(&path) {
                                let mut buffer = Vec::new();

                                // Read the file into the buffer
                                if file.read_to_end(&mut buffer).is_ok() {
                                    if let Ok(custom_image) = decode(&buffer) {
                                        self.handle_image_load(&custom_image, view_size);
                                    } else {
                                        eprintln!("Failed to decode custom image format");
                                    }
                                } else {
                                    eprintln!("Failed to read the file into buffer");
                                }
                            } else {
                                eprintln!("Failed to open file: {:?}", path);
                            }
                        }

                        if ext == "png"
                            || ext == "jpg"
                            || ext == "jpeg"
                            || ext == "bmp"
                            || ext == "webp"
                        {
                            if let Ok(img) = image::open(&path) {
                                self.handle_image_load(&img, view_size);
                            }
                        }
                    }
                }
            }

            if ui.button("Save Image").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    if let Some(path_str) = path.to_str() {
                        match File::create(path_str) {
                            Ok(mut file) => {
                                let ((x_min, y_min), (x_max, y_max)) = self.image.bounds.unwrap();

                                let width = (x_max - x_min + 1) as u16;
                                let height = (y_max - y_min + 1) as u16;

                                let rgba_data = self
                                    .image
                                    .chunks
                                    .iter()
                                    .flat_map(|c| c.1.pixels.iter())
                                    .copied()
                                    .flat_map(|p| vec![p.r(), p.g(), p.b(), p.a()])
                                    .collect::<Vec<u8>>();

                                let data = encode(width, height, &rgba_data as &[u8]).unwrap();

                                if let Err(e) = file.write_all(&data) {
                                    eprintln!("Failed to write data to file: {}", e);
                                } else {
                                    println!("File saved successfully to {}", path_str);
                                }
                            }
                            Err(e) => eprintln!("Failed to create file: {}", e),
                        }
                    } else {
                        eprintln!("Invalid file path.");
                    }
                } else {
                    println!("Save operation canceled.");
                }
            }

            ui.separator();

            // Zoom controls
            if ui.button("🔍 Fit to View").clicked() {
                self.zoom = self.calculate_zoom_to_fit(view_size);
                self.center_view(view_size);
            }

            if ui.button("⚖️ 100%").clicked() {
                self.zoom = 1.0;
                self.center_view(view_size);
            }

            // Zoom percentage display and manual input
            let mut zoom_text = format!("{:.0}%", self.zoom * 100.0);
            if ui.text_edit_singleline(&mut zoom_text).lost_focus() {
                if let Ok(percentage) = zoom_text.trim().trim_end_matches('%').parse::<f32>() {
                    self.zoom = (percentage / 100.0).clamp(0.1, 10.0);
                    self.center_view(view_size);
                }
            }

            ui.separator();

            // Grid toggle
            ui.checkbox(&mut self.draw_grid, "🔲 Show Grid");
        });
    }

    fn draw_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("color_panel").show(ctx, |ui| {
            ui.heading("Color Palette");
            ui.add_space(8.0);

            // Current color display
            ui.label("Current Color:");
            let color_size = Vec2::new(ui.available_width(), 30.0);
            let mut colorPickerColor = self.colors[self.current_color];

            // Using allocate_ui_with_layout to control the layout and size of the color picker
            ui.allocate_ui_with_layout(
                color_size,
                Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.color_edit_button_srgba(&mut colorPickerColor);
                },
            );

            if colorPickerColor != self.colors[self.current_color] {
                self.colors[self.current_color] = colorPickerColor;
            }
            ui.add_space(8.0);

            // Default color palette
            ui.label("Colors:");
            ui.add_space(4.0);

            for chunk in self.colors.chunks(4) {
                ui.horizontal(|ui| {
                    for (i, &color) in chunk.iter().enumerate() {
                        let size = Vec2::new(30.0, 30.0);

                        // Check if this color's index matches the current selected color index
                        let stroke = if self.current_color == i {
                            egui::Stroke::new(4.0, Color32::WHITE)
                        } else {
                            egui::Stroke::new(1.0, Color32::WHITE)
                        };

                        if ui
                            .add(
                                egui::Button::new("")
                                    .fill(color)
                                    .min_size(size)
                                    .stroke(stroke),
                            )
                            .clicked()
                        {
                            // Set the current color index to the clicked color's index in self.colors
                            self.current_color = i;
                        }
                    }
                });
            }

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.set_height(30.0);
                ui.set_width(200.0);

                // Add and Remove buttons centered within the container
                ui.with_layout(
                    Layout::from_main_dir_and_cross_align(
                        egui::Direction::LeftToRight,
                        Align::Center,
                    ),
                    |ui| {
                        if ui.add(egui::Button::new("Add")).clicked() {
                            self.colors.push(colorPickerColor);
                            self.current_color = self.colors.len() - 1;
                        }

                        if ui.button("Remove").clicked() {
                            self.colors.remove(self.current_color);
                            ()
                        }
                    },
                );
            });

            ui.add_space(8.0);
            if ui.button("Clear Canvas").clicked() {
                self.image.clear();
                self.cached_visible_chunks.clear();
            }

            // Instructions
            ui.add_space(16.0);
            ui.label("Controls:");
            ui.label("• Left click to draw");
            ui.label("• Middle click to pan");
            ui.label("• Ctrl + Scroll to zoom");
        });
    }
}

impl eframe::App for PixelEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.draw_side_panel(ctx);
        self.draw_central_panel(ctx);
    }
}
