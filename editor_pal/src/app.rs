use std::fs::File;
use std::io::Read;

use eframe::egui::{self, Layout};
use eframe::Frame;
use egui::{Color32, Pos2, Rect, Vec2};

use crate::canvas::{self, Canvas, CELL_SIZE};
use crate::chunk::CHUNK_SIZE;
use crate::filemanager;
use crate::image_source::ImageSource;
use crate::palette::Palette;
use crate::viewport::{update_canvas_viewport, ViewportInfo, ViewportOptions};

pub fn grid_to_screen(viewport_info: &ViewportInfo, grid_pos: (u32, u32)) -> (f32, f32) {
    let cell_size = viewport_info.zoom() * CELL_SIZE as f32;
    let x = grid_pos.0 as f32 * cell_size;
    let y = grid_pos.1 as f32 * cell_size;

    let min = viewport_info.get_parent_rect().unwrap().min;
    let adjusted_pos = min + Vec2::new(x, y);

    (adjusted_pos.x, adjusted_pos.y)
}

fn screen_to_grid(viewport_info: &ViewportInfo, canvas_dims: &(u32, u32), pos: Pos2) -> (u32, u32) {
    let viewport = viewport_info.get_parent_rect().unwrap();
    let pos = pos.to_vec2() - viewport.min.to_vec2();

    let cell_size = viewport_info.zoom() * CELL_SIZE as f32;
    let pos = Vec2::new((pos.x / cell_size).floor(), (pos.y / cell_size).floor());

    let pos = pos.clamp(
        Vec2::ZERO,
        Vec2::new(canvas_dims.0 as f32 - 1.0, canvas_dims.1 as f32 - 1.0),
    );

    (pos.x as u32, pos.y as u32)
}

#[derive(Default)]
pub struct PixelEditor {
    canvas: Canvas,
    viewport_info: ViewportInfo,
    viewport_options: ViewportOptions,
    palette: Palette,
}

impl PixelEditor {
    pub fn new() -> Self {
        Self {
            canvas: Canvas::new(32, 32),
            viewport_info: ViewportInfo::new(),
            viewport_options: ViewportOptions::new(),
            palette: Palette::new(),
        }
    }

    fn get_visible_chunk_indexes(&self, rect: &Rect) -> Vec<(u32, u32)> {
        let canvas_dims = self.canvas.dimensions();
        let top_left = screen_to_grid(&self.viewport_info, &canvas_dims, rect.min);
        let bottom_right = screen_to_grid(&self.viewport_info, &canvas_dims, rect.max);

        let chunk_size = CHUNK_SIZE as u32;

        let chunk_start_x = top_left.0.div_euclid(chunk_size);
        let chunk_start_y = top_left.1.div_euclid(chunk_size);
        let chunk_end_x = bottom_right.0.div_euclid(chunk_size);
        let chunk_end_y = bottom_right.1.div_euclid(chunk_size);

        let mut chunks = Vec::new();
        for chunk_y in chunk_start_y..=chunk_end_y {
            for chunk_x in chunk_start_x..=chunk_end_x {
                chunks.push((chunk_x, chunk_y));
            }
        }
        chunks
    }

    fn handle_image_load<T: ImageSource>(&mut self, source: &T) {
        self.canvas.load_image(source);

        // Center the view on the loaded image and adjust zoom
        // self.center_view(view_size);
        // self.zoom = self.calculate_zoom_to_fit(view_size);
    }

    fn draw_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = ui.allocate_response(ui.available_size(), egui::Sense::drag());
            let visible_rect = response.rect;

            let input = ui.input(|i| i.clone());

            let canvas_dims = self.canvas.dimensions();

            let image_size = Vec2::new(
                canvas_dims.0 as f32 * self.viewport_info.zoom() * 20.0,
                canvas_dims.1 as f32 * self.viewport_info.zoom() * 20.0,
            );
            let image_rect = egui::Rect::from_center_size(
                ui.max_rect().center() + self.viewport_info.pan_offset(),
                image_size,
            );

            self.viewport_info
                .update(image_rect, visible_rect, input.pointer.hover_pos());

            let painter = ui.painter();

            update_canvas_viewport(
                &input,
                &mut self.viewport_info,
                &mut self.viewport_options,
                &canvas_dims,
            );

            let visible_chunks = self.get_visible_chunk_indexes(&visible_rect);

            // Update texture cache
            let cached_chunks = self.canvas.update_cache(&visible_chunks, &ctx);

            // Draw the chunks as before, using the updated pan_offset
            for &chunk_pos in &visible_chunks {
                if let Some(texture) = cached_chunks.get(&chunk_pos) {
                    let (x, y) = chunk_pos;
                    let absolute_pos = canvas::Canvas::get_absolute_coords(x, y, 0, 0);
                    let absolute_pos_end =
                        canvas::Canvas::get_absolute_coords(x, y, CHUNK_SIZE, CHUNK_SIZE);

                    let start = grid_to_screen(&self.viewport_info, absolute_pos);
                    let end = grid_to_screen(&self.viewport_info, absolute_pos_end);

                    let chunk_rect =
                        Rect::from_two_pos(Pos2::new(start.0, start.1), Pos2::new(end.0, end.1));

                    if chunk_rect.intersects(visible_rect) {
                        painter.image(
                            texture.id(),
                            chunk_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }

                    painter.rect_stroke(chunk_rect, 0.0, egui::Stroke::new(0.5, Color32::RED));
                }
            }

            // Draw grid if enabled and zoom level is high enough
            if self.viewport_options.draw_grid && visible_chunks.len() < 5 {
                let top_left = screen_to_grid(&self.viewport_info, &canvas_dims, visible_rect.min);
                let bottom_right =
                    screen_to_grid(&self.viewport_info, &canvas_dims, visible_rect.max);

                for x in top_left.0..=bottom_right.0 {
                    for y in top_left.1..=bottom_right.1 {
                        let cell_min = grid_to_screen(&self.viewport_info, (x, y));
                        let cell_max = grid_to_screen(&self.viewport_info, (x + 1, y + 1));

                        let cell_min = Pos2::new(cell_min.0, cell_min.1);
                        let cell_max = Pos2::new(cell_max.0, cell_max.1);

                        painter.rect_stroke(
                            Rect::from_min_max(cell_min, cell_max),
                            0.0,
                            egui::Stroke::new(0.4, Color32::DARK_GRAY),
                        );
                    }
                }
            }

            // Handle drawing on the canvas
            if response.clicked() || (response.dragged() && input.pointer.primary_down()) {
                if let Some(pos) = input.pointer.hover_pos() {
                    let grid_pos = screen_to_grid(&self.viewport_info, &canvas_dims, pos);
                    let _ = self.canvas.set_pixel(
                        grid_pos.0,
                        grid_pos.1,
                        self.palette.get_current_color(),
                    );
                }
            }
        });
    }

    fn draw_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Load image button
                if ui.button("📂 Load Image").clicked() {
                    let image = filemanager::open_image()?;
                    self.handle_image_load(&*image?);
                }

                if ui.button("Save Image").clicked() {
                    let (width, height) = self.canvas.dimensions();
                    let rgba_data = self.canvas.get_data();
                    filemanager::save_image((width, height), rgba_data);
                }

                // ui.separator();

                // // Zoom controls
                // if ui.button("🔍 Fit to View").clicked() {
                //     self.zoom = self.calculate_zoom_to_fit(view_size);
                //     self.center_view(view_size);
                // }

                // if ui.button("⚖️ 100%").clicked() {
                //     self.zoom = 1.0;
                //     self.center_view(view_size);
                // }

                // Zoom percentage display and manual input
                // let mut zoom_text = format!("{:.0}%", self.zoom * 100.0);
                // if ui.text_edit_singleline(&mut zoom_text).lost_focus() {
                //     if let Ok(percentage) = zoom_text.trim().trim_end_matches('%').parse::<f32>() {
                //         self.zoom = (percentage / 100.0).clamp(0.1, 10.0);
                //         self.center_view(view_size);
                //     }
                // }

                ui.separator();

                // Grid toggle
                ui.checkbox(&mut self.viewport_options.draw_grid, "🔲 Show Grid");
            });
        });
    }

    fn draw_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("color_panel")
            .resizable(false) // Disable resizing
            .min_width(200.0) // Set minimum width to the fixed size
            .max_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Color Palette");
                ui.add_space(8.0);

                // Current color display
                ui.label("Current Color:");
                let color_size = Vec2::new(ui.available_width(), 30.0);
                let mut picker_color = self.palette.get_current_color();

                // Using allocate_ui_with_layout to control the layout and size of the color picker
                ui.allocate_ui_with_layout(
                    color_size,
                    Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.color_edit_button_srgba(&mut picker_color);
                    },
                );

                if picker_color != self.palette.get_current_color() {
                    self.palette.set_current_color(picker_color);
                }
                ui.add_space(8.0);

                // Default color palette
                ui.label("Colors:");
                ui.add_space(4.0);

                let color_size = Vec2::new(30.0, 30.0);
                let colors_total = self.palette.get_palette_length();
                let current_index = self.palette.get_current_color_index();

                for row in 0..(colors_total / 4) + 1 {
                    ui.horizontal(|ui| {
                        for col in 0..4 {
                            let index = row * 4 + col;

                            if index > colors_total {
                                break;
                            }

                            if index == colors_total {
                                if ui
                                    .add(egui::Button::new("Add").min_size(color_size))
                                    .clicked()
                                {
                                    self.palette.add_color(Color32::WHITE);
                                }
                                break;
                            }

                            let stroke = if current_index == index {
                                egui::Stroke::new(4.0, Color32::WHITE)
                            } else {
                                egui::Stroke::new(1.0, Color32::WHITE)
                            };

                            let color = self.palette.get_color(index).unwrap();

                            if ui
                                .add(
                                    egui::Button::new("")
                                        .fill(color)
                                        .stroke(stroke)
                                        .min_size(color_size),
                                )
                                .clicked()
                            {
                                self.palette.switch_color(index);
                            }
                        }
                    });
                }

                ui.add_space(8.0);

                if ui.button("Remove").clicked() {
                    self.palette.remove_color()
                }

                ui.add_space(8.0);
                if ui.button("Clear Canvas").clicked() {
                    self.canvas.clear();
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
        self.draw_central_panel(ctx);
        self.draw_side_panel(ctx);
        self.draw_toolbar(ctx);
    }
}
