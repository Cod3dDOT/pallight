use eframe::egui::Color32;

#[derive(Default)]
pub struct Palette {
    colors: Vec<Color32>,
    current_color: usize,
}

impl Palette {
    pub fn new() -> Self {
        Self {
            colors: vec![Color32::WHITE],
            current_color: 0,
        }
    }

    pub fn get_palette_length(&self) -> usize {
        self.colors.len()
    }

    pub fn add_color(&mut self, color: Color32) {
        if self.colors.len() < 256 {
            self.colors.push(color);
        }
    }

    pub fn remove_color(&mut self) {
        if self.colors.len() > 1 {
            self.colors.remove(self.current_color);
        }
    }

    pub fn get_current_color(&self) -> Color32 {
        self.get_color(self.current_color).unwrap()
    }

    pub fn get_current_color_index(&self) -> usize {
        self.current_color
    }

    pub fn set_current_color(&mut self, color: Color32) {
        self.set_color(self.current_color, color);
    }

    pub fn get_color(&self, index: usize) -> Option<Color32> {
        if index >= self.colors.len() {
            return None;
        }
        Some(self.colors[index])
    }

    pub fn set_color(&mut self, index: usize, color: Color32) -> Option<usize> {
        if index >= self.colors.len() {
            return None;
        }

        self.colors[index] = color;
        Some(index)
    }

    pub fn switch_color(&mut self, index: usize) {
        if index >= self.colors.len() {
            return;
        }
        self.current_color = index;
    }
}
