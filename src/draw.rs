use std::f32::consts::PI;

use egui::Color32;

use crate::App;

impl App {
    pub fn set_px(&mut self, x: usize, y: usize, px: u32) {
        let size = self.image.size();
        if size[0] <= x || size[1] <= y {
            return; // just ignore
        }
        self.image[[x, y]] = Color32::from_rgb((px >> 16) as u8, (px >> 8) as u8, px as u8);
        self.changes.push(x, y);
    }

    pub fn draw_dot(&mut self, x: usize, y: usize) {
        let color = self.color.into_color();
        self.set_px(x, y + 1, color);
        self.set_px(x.wrapping_sub(1), y, color);
        self.set_px(x, y, color);
        self.set_px(x + 1, y, color);
        self.set_px(x, y.wrapping_sub(1), color);
    }

    pub fn draw_line(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        func: fn(&mut Self, usize, usize),
    ) {
        let dx = x2 as f32 - x1 as f32;
        let dy = y2 as f32 - y1 as f32;
        let dist = (dx * dx + dy * dy).sqrt();
        let step_x = dx / dist;
        let step_y = dy / dist;
        let mut fx = x1 as f32;
        let mut fy = y1 as f32;
        for _ in 0..(dist + 1.0) as usize {
            func(self, fx as usize, fy as usize);
            if fx as usize == x2 && fy as usize == y2 {
                break;
            }
            fx += step_x;
            fy += step_y;
        }
    }

    pub fn draw_mouse(&mut self, x: usize, y: usize, func: fn(&mut Self, usize, usize)) {
        let [last_x, last_y] = self.last_mouse_pos.unwrap_or([x, y]);
        self.draw_line(last_x, last_y, x, y, func);
        self.last_mouse_pos = Some([x, y]);
    }

    pub fn draw_ngon(&mut self, x: usize, y: usize, n: usize, radius: f32, begin_angle: f32) {
        let begin_angle = (begin_angle - 90.0) / 180.0 * PI;
        let angle_increment = PI * 2.0 / n as f32;
        let mut current_angle = angle_increment + begin_angle;
        let fx = x as f32;
        let fy = y as f32;
        let mut last_x = begin_angle.cos() * radius;
        let mut last_y = begin_angle.sin() * radius;
        for _ in 0..n {
            let new_x = current_angle.cos() * radius;
            let new_y = current_angle.sin() * radius;
            self.draw_line(
                (fx + last_x) as usize,
                (fy + last_y) as usize,
                (fx + new_x) as usize,
                (fy + new_y) as usize,
                Self::draw_dot,
            );
            last_x = new_x;
            last_y = new_y;
            current_angle += angle_increment;
        }
    }
}
