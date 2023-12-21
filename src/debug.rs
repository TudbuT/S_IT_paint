use std::time::SystemTime;

use crate::App;

pub struct Effects {
    rand: u64,
    pub randomize_size: bool,
    pub checkerboard: bool,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            rand: SystemTime::UNIX_EPOCH.elapsed().unwrap().as_micros() as u64,
            randomize_size: false,
            checkerboard: false,
        }
    }
}

// bad (but good-enough for this purpose) random number generator
fn rand(rand: &mut u64) -> u8 {
    let state = *rand;
    let r = ((state & 0b1111) << 60) + ((state >> 60) & 0b1111);
    *rand = state.rotate_right(1) ^ r;
    r as u8
}

impl App {
    pub fn update_effects(&mut self) {
        if self.effects.randomize_size {
            self.draw.size = self
                .draw
                .size
                .saturating_add_signed(rand(&mut self.effects.rand) as isize % 7 - 3);
        }
    }
}
