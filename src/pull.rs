use egui::{InputState, Pos2, Rect};

use crate::App;

impl App {
    pub fn sync(&mut self) {
        self.real_image = self.image.clone();
    }

    // not perfectly efficient, but fast enough to be responsive
    pub fn pull(&mut self, inp: &InputState, pointer_pos: [usize; 2]) {
        if let Some(pull_start) = self.pull_start {
            // clone the image to reset it, then draw the current state of the pulled brush
            // is this inefficient? yes.
            self.image = self.real_image.clone();
            let img_size = self.image.size();
            self.changes.all(Rect::from_min_max(
                Pos2::ZERO,
                Pos2::new(img_size[0] as f32, img_size[1] as f32),
            ));

            // the distance pulled divided by two (-> the radius)
            let pull_x = (pointer_pos[0] as isize - pull_start[0] as isize) / 2;
            let pull_y = (pointer_pos[1] as isize - pull_start[1] as isize) / 2;
            let pull_size = if inp.modifiers.shift {
                // if shift is pressed, both sizes are the same (specifically, the biggest of the two)
                #[inline]
                fn sign(x: isize) -> isize {
                    if x < 0 {
                        -1
                    } else {
                        1
                    }
                }
                // preserve the sign despite the absolute max() call
                let x = pull_x.abs().max(pull_y.abs());
                [x * sign(pull_x), x * sign(pull_y)]
            } else {
                [pull_x, pull_y]
            };

            // draw:
            self.mode
                .into_fn_sized(pull_size[0] as f32, pull_size[1] as f32)(
                self,
                self.draw.at(
                    // this is going to be the center
                    (pull_start[0] as isize + pull_size[0]) as usize,
                    (pull_start[1] as isize + pull_size[1]) as usize,
                ),
            );

            // reset and save the pull if user has stopped pulling
            if !inp.pointer.secondary_down() {
                self.pull_start = None;
                self.sync();
            }
        } else {
            // start a pull
            self.pull_start = Some(pointer_pos);
            self.sync();
        }
    }
}
