use std::sync::Arc;

use egui::*;
use epaint::{ImageDelta, TextureManager};
use micro_ndarray::Array;

use crate::App;

impl App {
    pub fn correct_tex_size(&mut self, texman: &mut TextureManager, window_size: [usize; 2]) {
        if self.image.size()[1..3] == window_size {
            return;
        }
        let mut new_image = Array::new_with([3, window_size[0], window_size[1]], 0xff);
        for (pos, pixel) in self.image.iter() {
            if let Some(px) = new_image.get_mut(pos) {
                *px = *pixel;
            }
        }
        self.image = new_image;

        let cimg = ColorImage::from_rgb(
            self.image.size()[1..3].try_into().unwrap(),
            self.image.as_flattened(),
        );
        texman.free(self.tex);
        self.tex = texman.alloc(
            "canvas".to_owned(),
            ImageData::Color(Arc::new(cimg)),
            TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Linear,
            },
        );
        self.image_to_texture(texman);
    }

    pub fn image_to_texture(&mut self, texman: &mut TextureManager) {
        let cimg = ColorImage::from_rgb(
            self.image.size()[1..3].try_into().unwrap(),
            self.image.as_flattened(),
        );
        texman.set(
            self.tex,
            ImageDelta::full(
                cimg,
                TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Linear,
                },
            ),
        );
    }
}
