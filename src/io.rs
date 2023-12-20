use egui::{Color32, Pos2, Rect};
use image::{io::Reader as ImageReader, DynamicImage, ImageBuffer, Rgb};
use micro_ndarray::Array;

use crate::App;

impl App {
    /// SAFETY: Call only when self.filename is present
    /// loads a file from disk (called after open dialog is confirmed)
    pub fn load(&mut self) {
        if let Ok(x) = ImageReader::open(self.filename.as_ref().unwrap())
            .expect("This file can't be opened due to an IO error")
            .decode()
        {
            self.image = Array::from_flat(
                x.to_rgb8()
                    .pixels()
                    .map(|&Rgb([r, g, b])| Color32::from_rgb(r, g, b))
                    .collect::<Vec<_>>(),
                [x.width() as usize, x.height() as usize],
            )
            .unwrap();
            self.changes.all(Rect::from_min_max(
                Pos2::ZERO,
                Pos2::new(x.width() as f32, x.height() as f32),
            ));
        } else {
            self.filename = None;
            println!("Unable to load this image.");
        }
    }

    /// SAFETY: Call only when self.filename is present
    /// saves the image to disk
    pub fn save(&mut self) {
        let size = self.image.size();
        DynamicImage::ImageRgb8(ImageBuffer::from_fn(
            size[0] as u32,
            size[1] as u32,
            |x, y| {
                let px: Color32 = self.image[[x as usize, y as usize]];
                Rgb(px.to_array()[0..3].try_into().unwrap())
            },
        ))
        .save(self.filename.as_ref().unwrap())
        .expect("This file can't be saved to due to an IO error");
    }
}
