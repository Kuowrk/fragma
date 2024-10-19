use std::collections::HashMap;
use image::RgbaImage;

#[derive(Default)]
pub struct Assets {
    images: HashMap<String, RgbaImage>,
}

impl Assets {
    pub fn add_image(&mut self, name: String, image: RgbaImage) {
        self.images.insert(name, image);
    }

    pub fn get_image(&self, name: &str) -> Option<&RgbaImage> {
        self.images.get(name)
    }
}