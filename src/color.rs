use egui::*;

use crate::App;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DrawColor {
    Black,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    Brown,
    Aqua,
    Purple,
}

use DrawColor::*;

impl DrawColor {
    pub fn menu(app: &mut App, ui: &mut Ui) {
        ui.radio_value(&mut app.color, Black, "Black");
        ui.radio_value(&mut app.color, White, "White");
        ui.radio_value(&mut app.color, Red, "Red");
        ui.radio_value(&mut app.color, Green, "Green");
        ui.radio_value(&mut app.color, Blue, "Blue");
        ui.radio_value(&mut app.color, Yellow, "Yellow");
        ui.radio_value(&mut app.color, Orange, "Orange");
        ui.radio_value(&mut app.color, Brown, "Brown");
        ui.radio_value(&mut app.color, Aqua, "Aqua");
        ui.radio_value(&mut app.color, Purple, "Purple");
    }

    pub fn into_color(self) -> u32 {
        match self {
            Black => 0,
            White => 0xffffff,
            Red => 0xff0000,
            Green => 0x00ff00,
            Blue => 0x0000ff,
            Yellow => 0xffff00,
            Orange => 0xff8000,
            Brown => 0x654321,
            Aqua => 0x00ffff,
            Purple => 0xff00ff,
        }
    }
}
