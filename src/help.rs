use egui::{vec2, Label, RichText, Ui};

use crate::App;

impl App {
    pub(crate) fn render_help(&mut self, ui: &mut Ui) {
        ui.add_sized(vec2(300.0, 30.0), Label::new(RichText::new("You can select tools and colors in the window menu. \nTo draw the shapes with arbitrary sizes, use the right mouse button and hold shift to draw precise squares / equilateral triangles / circles.")));
    }
}
