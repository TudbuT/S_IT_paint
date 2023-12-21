//! These are quite complex, but I will do my best to explain them anyway.

use std::f32::consts::PI;

use egui::Color32;

use crate::App;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

impl Location {
    #[inline]
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn offset(&self, x: isize, y: isize) -> Self {
        Self {
            x: self.x.wrapping_add_signed(x),
            y: self.y.wrapping_add_signed(y),
        }
    }

    #[inline]
    pub fn at(&self, x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DrawParams {
    pub loc: Location,
    pub size: usize,
    pub px: u32,
}

impl DrawParams {
    #[inline]
    pub fn new(x: usize, y: usize, size: usize, px: u32) -> Self {
        Self {
            loc: Location { x, y },
            size,
            px,
        }
    }

    #[inline]
    pub fn offset(&self, x: isize, y: isize) -> Self {
        Self {
            loc: self.loc.offset(x, y),
            ..*self
        }
    }

    #[inline]
    pub fn at(&self, x: usize, y: usize) -> Self {
        Self {
            loc: self.loc.at(x, y),
            ..*self
        }
    }

    #[inline]
    pub fn at_sized(&self, x: usize, y: usize, size: usize) -> Self {
        Self {
            loc: self.loc.at(x, y),
            size,
            ..*self
        }
    }
}

impl App {
    pub fn set_px(&mut self, draw: DrawParams) {
        let DrawParams {
            loc: Location { x, y },
            px,
            ..
        } = draw;
        let size = self.image.size();
        if x >= size[0] || y >= size[1] {
            return; // just ignore
        }
        if self.effects.checkerboard && (x + y) % 2 == 0 {
            return;
        }
        self.image[[x, y]] = Color32::from_rgb((px >> 16) as u8, (px >> 8) as u8, px as u8);
        self.changes.push(x, y);
    }

    /// Does not ignore pixels out of bounds, panic!s instead.
    pub fn set_px_unchecked(&mut self, x: usize, y: usize, col: Color32) {
        if self.effects.checkerboard && (x + y) % 2 == 0 {
            return;
        }
        self.image[[x, y]] = col;
        self.changes.push(x, y);
    }

    /// Draws a dot of arbitrary size. This is one px for a size of 0, a plus for size 1, and a rectangle of side length (size - 1) * 2
    pub fn draw_dot(&mut self, draw: DrawParams) {
        self.set_px(draw.offset(0, 0));

        if draw.size == 1 {
            self.set_px(draw.offset(0, 1));
            self.set_px(draw.offset(-1, 0));
            self.set_px(draw.offset(1, 0));
            self.set_px(draw.offset(0, -1));
        }

        if draw.size >= 2 {
            let size = draw.size as isize;
            for x in -size + 1..size {
                for y in -size + 1..size {
                    self.set_px(draw.offset(x, y));
                }
            }
        }
    }

    /// Draws a line made of the `func` argument by interpolating linearly between the two positions
    pub fn draw_line(
        &mut self,
        draw1: DrawParams,
        draw2: DrawParams,
        func: fn(&mut Self, DrawParams),
    ) {
        assert_eq!(draw1.px, draw2.px, "Cannot change colors mid-line");
        let DrawParams {
            loc: Location { x: x1, y: y1 },
            size: size1,
            ..
        } = draw1;
        let DrawParams {
            loc: Location { x: x2, y: y2 },
            size: size2,
            ..
        } = draw2;
        let dx = x2 as f32 - x1 as f32; // the offset in x direction
        let dy = y2 as f32 - y1 as f32; // the offset in y direction
        let dsize = size2 as f32 - size1 as f32; // the change in size over the distance
        let dist = (dx * dx + dy * dy).sqrt(); // the distance
        let step_x = dx / dist; // the change of x over a distance of 1 pixel
        let step_y = dy / dist; // the change of y over a distance of 1 pixel
        let step_size = dsize / dist; // the change in size over a distance of 1 pixel
                                      // the values as floats
        let mut fx = x1 as f32;
        let mut fy = y1 as f32;
        let mut fsize = size1 as f32;
        // loop until distance is reached, but overshoot
        for _ in 0..(dist + 1.0) as usize {
            // draw
            func(
                self,
                draw1.at_sized(fx as usize, fy as usize, fsize.round() as usize),
            );
            // modify values by the needed amount
            fx += step_x;
            fy += step_y;
            fsize += step_size;
            // if arrived at destination, stop (this is why the overshoot is not a problem)
            if fx as usize == x2 && fy as usize == y2 {
                break;
            }
        }
    }

    pub fn draw_mouse(&mut self, draw: DrawParams, func: fn(&mut Self, DrawParams)) {
        let last = self.last_mouse_pos.unwrap_or(draw);
        self.draw_line(last, draw, func);
        self.last_mouse_pos = Some(draw);
    }

    /// Draws an n-gon (polygon) with an arbitrary rotation, radius, and amount of corners (n)
    /// by drawing around a center point at angles in increments of π*2 / n
    pub fn draw_ngon(
        &mut self,
        draw: DrawParams,
        mut n: usize,
        mut radius_x: f32,
        mut radius_y: f32,
        begin_angle: f32,
    ) {
        // n = 0 => draw a circle
        if n == 0 {
            n = (radius_x.abs().max(radius_y.abs()) * PI * 2.0) as usize; // circle can be approximated by having as many corners as pixels
        }

        // convert rotation to radians
        let begin_angle = begin_angle / 180.0 * PI /*start at top:*/ + PI;

        // to make the usual shapes feel more natural
        if n == 3 {
            radius_x /= (begin_angle + 2.0 / 3.0 * PI).sin();
        }
        if n == 4 {
            radius_x /= begin_angle.sin();
            radius_y /= begin_angle.cos();
        }

        // amount of radians between each corner if it were on a circle
        let angle_increment = PI * 2.0 / n as f32;

        // start one corner after the starting point because we draw lines
        let mut current_angle = angle_increment + begin_angle;

        // center
        let fx = draw.loc.x as f32;
        let fy = draw.loc.y as f32;

        let mut last_x = begin_angle.sin() * radius_x;
        let mut last_y = begin_angle.cos() * radius_y;
        // loop over corners and draw a line from the last to the current
        for _ in 0..n {
            let new_x = current_angle.sin() * radius_x;
            let new_y = current_angle.cos() * radius_y;

            self.draw_line(
                draw.at((fx + last_x) as usize, (fy + last_y) as usize),
                draw.at((fx + new_x) as usize, (fy + new_y) as usize),
                Self::draw_dot,
            );
            last_x = new_x;
            last_y = new_y;
            current_angle += angle_increment;
        }
    }
}
