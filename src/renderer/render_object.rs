use color_eyre::Result;
use crate::renderer::Camera;
use crate::renderer::resources::Resources;
use crate::renderer::viewport::Viewport;

pub struct RenderObject {
    material_name: String,
    texture_name: String,
    model_name: String,
}

impl RenderObject {
    pub fn new(
        material_name: String,
        texture_name: String,
        model_name: String,
    ) -> Self {
        Self {
            material_name,
            texture_name,
            model_name,
        }
    }

    pub fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass,
        camera: &mut Camera,
        resources: &Resources,
        viewport: &Viewport,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()> {
        let material = resources.get_render_material(&self.material_name)?;
        let texture = resources.get_texture(&self.texture_name)?;
        let model = resources.get_model(&self.model_name)?;

        render_pass.set_pipeline(material.get_pipeline());
        render_pass.set_bind_group(0, texture.get_bind_group(), &[]);
        render_pass.set_bind_group(1, camera.get_bind_group(
            viewport,
            device,
            queue,
        ), &[]);
        model.draw(render_pass);

        Ok(())
    }
}