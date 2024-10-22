use std::collections::HashMap;
use image::RgbaImage;

mod model;
use model::Model;

// Assets are more content-focused compared to Resources, which are more system-focused.
// Assets are usually loaded from disk or network, while Resources are usually created in memory.
#[derive(Default)]
pub struct Assets {
    models: HashMap<String, Model>,
    textures: HashMap<String, Texture>
}

impl Assets {
    pub fn add_image(&mut self, name: String, image: RgbaImage) {
        self.images.insert(name, image);
    }

    pub fn get_image(&self, name: &str) -> Option<&RgbaImage> {
        self.images.get(name)
    }
}