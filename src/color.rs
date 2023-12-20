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
        let mut clicked = false;
        // rust does not have reflection at the moment so this looks a bit ugly and repetitive
        // adds button to the menu
        clicked |= ui.radio_value(&mut app.color, Black, "Black").clicked();
        clicked |= ui.radio_value(&mut app.color, White, "White").clicked();
        clicked |= ui.radio_value(&mut app.color, Red, "Red").clicked();
        clicked |= ui.radio_value(&mut app.color, Green, "Green").clicked();
        clicked |= ui.radio_value(&mut app.color, Blue, "Blue").clicked();
        clicked |= ui.radio_value(&mut app.color, Yellow, "Yellow").clicked();
        clicked |= ui.radio_value(&mut app.color, Orange, "Orange").clicked();
        clicked |= ui.radio_value(&mut app.color, Brown, "Brown").clicked();
        clicked |= ui.radio_value(&mut app.color, Aqua, "Aqua").clicked();
        clicked |= ui.radio_value(&mut app.color, Purple, "Purple").clicked();
        // if any have been clicked, change the color to it
        if clicked {
            app.draw.px = app.color.into_color();
        }
    }
}

pub trait ColorConvert {
    fn into_color(self) -> u32;
    fn into_colorf(self) -> [f32; 3];
}

impl ColorConvert for DrawColor {
    fn into_color(self) -> u32 {
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

    fn into_colorf(self) -> [f32; 3] {
        self.into_color().into_colorf()
    }
}

impl ColorConvert for u32 {
    fn into_color(self) -> u32 {
        self
    }

    fn into_colorf(self) -> [f32; 3] {
        [
            (self >> 16 & 0xff) as f32 / 255.0,
            (self >> 8 & 0xff) as f32 / 255.0,
            (self & 0xff) as f32 / 255.0,
        ]
    }
}

impl ColorConvert for [f32; 3] {
    fn into_color(self) -> u32 {
        let r = (self[0] * 255.0) as u32;
        let g = (self[1] * 255.0) as u32;
        let b = (self[2] * 255.0) as u32;
        (r << 16) + (g << 8) + b
    }

    fn into_colorf(self) -> [f32; 3] {
        self
    }
}
