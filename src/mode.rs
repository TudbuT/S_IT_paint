use egui::*;

use crate::{draw::DrawParams, App};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Paintbrush,
    Triangle,
    Square,
    Circle,
    Fill,
}

use Mode::*;

impl Mode {
    pub fn menu(app: &mut App, ui: &mut Ui) {
        ui.radio_value(&mut app.mode, Paintbrush, "Paintbrush");
        ui.radio_value(&mut app.mode, Triangle, "Triangle");
        ui.radio_value(&mut app.mode, Square, "Square");
        ui.radio_value(&mut app.mode, Circle, "Circle");
        ui.radio_value(&mut app.mode, Fill, "Fill");
    }

    /// Some things shouldn't be interpolated and only run once
    pub fn run_once(self) -> bool {
        #[allow(clippy::match_like_matches_macro)] // this may have more added later
        match self {
            Fill => true,
            _ => false,
        }
    }

    /// universalizes the mode into a single function with pre-set size
    pub fn into_fn_sized(self, radius_x: f32, radius_y: f32) -> fn(&mut App, DrawParams) {
        static mut RADIUS: (f32, f32) = (0.0, 0.0);
        // this is single-threaded so a static is fine
        unsafe {
            RADIUS = (radius_x, radius_y); // need this for compatible match arms
            match self {
                Paintbrush => App::draw_dot,
                Triangle => |this, draw| this.draw_ngon(draw, 3, RADIUS.0, -RADIUS.1, 180.0),
                Square => |this, draw| this.draw_ngon(draw, 4, RADIUS.0, -RADIUS.1, 45.0),
                Circle => |this, draw| this.draw_ngon(draw, 0, RADIUS.0, -RADIUS.1, 0.0),
                Fill => |this, draw| this.fill(draw),
            }
        }
    }

    /// universalizes the mode into a single function
    pub fn into_fn(self) -> fn(&mut App, DrawParams) {
        match self {
            Paintbrush => App::draw_dot,
            Triangle => |this, draw| {
                this.draw_ngon(
                    draw,
                    3,
                    this.draw.size.max(1) as f32 * 30.0,
                    this.draw.size.max(1) as f32 * 30.0,
                    0.0,
                )
            },
            Square => |this, draw| {
                this.draw_ngon(
                    draw,
                    4,
                    this.draw.size.max(1) as f32 * 30.0,
                    this.draw.size.max(1) as f32 * 30.0,
                    45.0,
                )
            },
            Circle => |this, draw| {
                this.draw_ngon(
                    draw,
                    0,
                    this.draw.size.max(1) as f32 * 30.0,
                    this.draw.size.max(1) as f32 * 30.0,
                    0.0,
                )
            },
            Fill => |this, draw| this.fill(draw),
        }
    }
}
