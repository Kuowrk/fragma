use std::cell::RefCell;
use std::rc::Rc;
use color_eyre::eyre::Result;
use crate::renderer::render_object::RenderObject;
use crate::renderer::resources::Resources;

pub struct Scene {
    render_objects: Vec<RenderObject>,
    resources: Rc<RefCell<Resources>>,
}

impl Scene {
    pub fn new(resources: Rc<RefCell<Resources>>) -> Self {
        Self {
            render_objects: Vec::new(),
            resources,
        }
    }

    pub fn get_render_objects(&self) -> &Vec<RenderObject> {
        &self.render_objects
    }

    pub fn add_render_object(&mut self, material_name: &str, texture_name: &str, model_name: &str) -> Result<()> {
        let resources = self.resources.try_borrow()?;
        let render_object = resources.create_render_object(material_name, texture_name, model_name)?;
        self.render_objects.push(render_object);
        Ok(())
    }
}