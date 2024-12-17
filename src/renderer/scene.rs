use std::cell::RefCell;
use std::rc::Rc;
use color_eyre::eyre::Result;
use crate::renderer::compute_object::ComputeObject;
use crate::renderer::render_object::RenderObject;
use crate::renderer::resources::Resources;

pub struct Scene {
    render_objects: Vec<RenderObject>,
    compute_objects: Vec<ComputeObject>,

    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    resources: Rc<RefCell<Resources>>,
}

impl Scene {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        resources: Rc<RefCell<Resources>>
    ) -> Self {
        Self {
            render_objects: Vec::new(),
            compute_objects: Vec::new(),

            device,
            queue,
            resources,
        }
    }

    pub fn get_render_objects(&self) -> &Vec<RenderObject> {
        &self.render_objects
    }

    pub fn get_compute_objects(&self) -> &Vec<ComputeObject> {
        &self.compute_objects
    }

    pub fn add_render_object(&mut self, material_name: &str, texture_name: &str, model_name: &str) -> Result<()> {
        let resources = self.resources.try_borrow()?;
        let render_object = resources.create_render_object(material_name, texture_name, model_name)?;
        self.render_objects.push(render_object);
        Ok(())
    }

    pub fn add_compute_object_with_output_texture(
        &mut self,
        material_name: &str,
        texture_width: u32,
        texture_height: u32,
    ) -> Result<()> {
        let resources = self.resources.try_borrow()?;
        let compute_object = resources.create_compute_object_with_output_texture(
            material_name,
            texture_width,
            texture_height,
            &self.device,
        )?;
        self.compute_objects.push(compute_object);
        Ok(())
    }

    pub fn resize_compute_output_textures(&mut self, width: u32, height: u32) -> Result<()> {
        for compute_object in self.compute_objects.iter_mut() {
            if compute_object.has_output_texture() {
                compute_object.resize_output_texture(
                    width,
                    height,
                    &self.device,
                    &self.queue,
                    &self.resources.borrow(),
                )?;
            }
        }
        Ok(())
    }
}