use std::time::SystemTime;

use crate::App;

pub struct Debug {
    rand: u64,
    pub randomize_size: bool,
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            rand: SystemTime::UNIX_EPOCH.elapsed().unwrap().as_micros() as u64,
            randomize_size: false,
        }
    }
}

fn rand(rand: &mut u64) -> u8 {
    let state = *rand;
    let r = ((state & 0b1111) << 60) + ((state >> 60) & 0b1111);
    *rand = state.rotate_right(1) ^ r;
    r as u8
}

impl App {
    pub fn update_debug(&mut self) {
        if self.debug.randomize_size {
            self.draw.size = self
                .draw
                .size
                .saturating_add_signed(rand(&mut self.debug.rand) as isize % 7 - 3);
        }
    }
}
