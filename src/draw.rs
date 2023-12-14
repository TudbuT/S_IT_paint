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
        self.image[[x, y]] = Color32::from_rgb((px >> 16) as u8, (px >> 8) as u8, px as u8);
        self.changes.push(x, y);
    }
    pub fn set_px_unchecked(&mut self, x: usize, y: usize, col: Color32) {
        self.image[[x, y]] = col;
        self.changes.push(x, y);
    }

    /// Draws a dot of arbitrary size (size 1 has no corners, all others do)
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
        let dx = x2 as f32 - x1 as f32;
        let dy = y2 as f32 - y1 as f32;
        let dsize = size2 as f32 - size1 as f32;
        let dist = (dx * dx + dy * dy).sqrt();
        let step_x = dx / dist;
        let step_y = dy / dist;
        let step_size = dsize / dist;
        let mut fx = x1 as f32;
        let mut fy = y1 as f32;
        let mut fsize = size1 as f32;
        for _ in 0..(dist + 1.0) as usize {
            func(
                self,
                draw1.at_sized(fx as usize, fy as usize, fsize.round() as usize),
            );
            if fx as usize == x2 && fy as usize == y2 {
                break;
            }
            fx += step_x;
            fy += step_y;
            fsize += step_size;
        }
    }

    pub fn draw_mouse(&mut self, draw: DrawParams, func: fn(&mut Self, DrawParams)) {
        let last = self.last_mouse_pos.unwrap_or(draw);
        self.draw_line(last, draw, func);
        self.last_mouse_pos = Some(draw);
    }

    /// draws a polygon (or circle) by drawing around a center point at angles in increments of π*2 / n
    pub fn draw_ngon(
        &mut self,
        draw: DrawParams,
        mut n: usize,
        mut radius_x: f32,
        mut radius_y: f32,
        begin_angle: f32,
    ) {
        if n == 0 {
            n = (radius_x.abs().max(radius_y.abs()) * PI) as usize;
        }
        let begin_angle = (begin_angle - 90.0) / 180.0 * PI;
        if n == 4 {
            radius_x /= begin_angle.cos();
            radius_y /= begin_angle.sin();
        }
        if n == 3 {
            radius_x /= (begin_angle + 2.0 / 3.0 * PI).cos();
        }
        let angle_increment = PI * 2.0 / n as f32;
        let mut current_angle = angle_increment + begin_angle;
        let fx = draw.loc.x as f32;
        let fy = draw.loc.y as f32;
        let mut last_x = begin_angle.cos() * radius_x;
        let mut last_y = begin_angle.sin() * radius_y;
        for _ in 0..n {
            let new_x = current_angle.cos() * radius_x;
            let new_y = current_angle.sin() * radius_y;
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
