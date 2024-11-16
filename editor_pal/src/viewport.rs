use eframe::egui::{InputState, Pos2, Rect, Vec2};

#[derive(Default)]
pub struct ViewportInfo {
    parent_rect: Option<Rect>,
    viewport: Option<Rect>,

    pan_offset: Vec2,
    target_pan_offset: Vec2,

    mouse_pos: Option<Pos2>,
    last_mouse_pos: Option<Pos2>,

    zoom: f32,
    target_zoom: f32,
}

impl ViewportInfo {
    pub fn new() -> Self {
        Self {
            parent_rect: None,
            mouse_pos: None,
            viewport: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            last_mouse_pos: None,
            target_zoom: 1.0,
            target_pan_offset: Vec2::ZERO,
        }
    }

    pub fn update(&mut self, parent_rect: Rect, viewport: Rect, mouse_pos: Option<Pos2>) {
        self.parent_rect = Some(parent_rect);
        self.viewport = Some(viewport);
        self.mouse_pos = mouse_pos;
    }

    pub fn pan_offset(&self) -> Vec2 {
        self.pan_offset
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn last_mouse_pos(&self) -> Option<Pos2> {
        self.last_mouse_pos
    }

    pub fn get_viewport(&self) -> Option<Rect> {
        self.viewport
    }

    pub fn get_parent_rect(&self) -> Option<Rect> {
        self.parent_rect
    }
}

#[derive(Default)]
pub struct ViewportOptions {
    pub draw_grid: bool,
}

impl ViewportOptions {
    pub fn new() -> Self {
        Self { draw_grid: true }
    }
}

pub fn update_zoom(input: &InputState, viewport_info: &mut ViewportInfo) {
    let zoom_speed = 0.1;
    viewport_info.zoom += (viewport_info.target_zoom - viewport_info.zoom) * zoom_speed;

    viewport_info.pan_offset +=
        (viewport_info.target_pan_offset - viewport_info.pan_offset) * zoom_speed;

    let scroll_delta = input.raw_scroll_delta.y;
    if scroll_delta == 0.0 {
        return;
    }

    let old_zoom = viewport_info.zoom;
    let zoom_delta = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
    viewport_info.target_zoom *= zoom_delta;
    viewport_info.target_zoom = viewport_info.target_zoom.clamp(0.01, 10.0);

    let viewport = viewport_info.viewport.unwrap();

    if let Some(mouse_pos) = input.pointer.hover_pos() {
        let relative_cursor_position = mouse_pos - viewport_info.pan_offset;
        let zoom_ratio = viewport_info.target_zoom / old_zoom;

        // Adjust pan_offset to keep the cursor position under the mouse
        viewport_info.target_pan_offset =
            mouse_pos - viewport.center() - relative_cursor_position.to_vec2() * zoom_ratio * 0.1;
    }
}

pub fn update_pan_offset(viewport_info: &mut ViewportInfo) {
    let mouse_pos = viewport_info.mouse_pos;

    if mouse_pos.is_none() {
        return;
    }

    let last_frame_pos = viewport_info.last_mouse_pos;

    if last_frame_pos.is_none() {
        viewport_info.last_mouse_pos = mouse_pos;
        return;
    }

    let mouse_pos = mouse_pos.unwrap();
    let last_frame_pos = last_frame_pos.unwrap();
    let viewport = viewport_info.viewport.unwrap();

    let delta = mouse_pos - last_frame_pos;
    viewport_info.pan_offset += delta;

    let boundaries = (
        -viewport.size().x / 2.0,
        viewport.size().x / 2.0,
        -viewport.size().y / 2.0,
        viewport.size().y / 2.0,
    );

    viewport_info.pan_offset.x = viewport_info.pan_offset.x.clamp(boundaries.0, boundaries.1);
    viewport_info.pan_offset.y = viewport_info.pan_offset.y.clamp(boundaries.2, boundaries.3);

    viewport_info.last_mouse_pos = Some(mouse_pos);
}

pub fn update_canvas_viewport(
    input: &InputState,
    viewport_info: &mut ViewportInfo,
    viewport_options: &mut ViewportOptions,
    canvas_dimensions: &(u32, u32),
) {
    if input.modifiers.ctrl {
        update_zoom(input, viewport_info);
    } else {
        viewport_info.target_zoom = viewport_info.zoom;
        viewport_info.target_pan_offset = viewport_info.pan_offset;
    }

    if input.pointer.middle_down() {
        update_pan_offset(viewport_info);
    } else {
        viewport_info.last_mouse_pos = None;
    }
}
