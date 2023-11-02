use image::{io::Reader as ImageReader, DynamicImage, ImageBuffer};
use micro_ndarray::Array;

use crate::App;

impl App {
    /// SAFETY: Call only when self.filename is present
    pub fn load(&mut self) {
        if let Ok(x) = ImageReader::open(self.filename.as_ref().unwrap())
            .expect("This file can't be opened due to an IO error")
            .decode()
        {
            self.image = Array::from_flat(
                x.to_rgb8().into_vec(),
                [3, x.width() as usize, x.height() as usize],
            )
            .unwrap();
        } else {
            self.filename = None;
            println!("Unable to load this image.");
        }
    }

    /// SAFETY: Call only when self.filename is present
    pub fn save(&mut self) {
        let size = self.image.size();
        DynamicImage::ImageRgb8(
            ImageBuffer::from_vec(
                size[1] as u32,
                size[2] as u32,
                self.image.clone().into_flattened(),
            )
            .unwrap(),
        )
        .save(self.filename.as_ref().unwrap())
        .expect("This file can't be saved to due to an IO error");
    }
}
