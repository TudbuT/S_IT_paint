use std::f32::consts::PI;

use egui::*;

use crate::App;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Paintbrush,
    Triangle,
    Square,
    Circle,
}

use Mode::*;

impl Mode {
    pub fn menu(app: &mut App, ui: &mut Ui) {
        ui.radio_value(&mut app.mode, Paintbrush, "Paintbrush");
        ui.radio_value(&mut app.mode, Triangle, "Triangle");
        ui.radio_value(&mut app.mode, Square, "Square");
        ui.radio_value(&mut app.mode, Circle, "Circle");
    }

    pub fn into_fn(self) -> fn(&mut App, usize, usize) {
        match self {
            Paintbrush => App::draw_dot,
            Triangle => |this, x, y| this.draw_ngon(x, y, 3, 30.0, 0.0),
            Square => |this, x, y| this.draw_ngon(x, y, 4, 30.0, 45.0),
            Circle => |this, x, y| this.draw_ngon(x, y, (30.0 * PI) as usize, 30.0, 0.0),
        }
    }
}
