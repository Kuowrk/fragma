use color_eyre::Result;
use crate::renderer::resources::Resources;

pub struct ComputeObject {
    material_name: String,
    texture_name: String,
}

impl ComputeObject {
    pub fn new(material_name: String, texture_name: String) -> Self {
        Self {
            material_name,
            texture_name,
        }
    }

    pub fn dispatch(
        &self,
        compute_pass: &mut wgpu::ComputePass,
        resources: &Resources,
    ) -> Result<()> {
        let material = resources.get_compute_material(&self.material_name)?;
        let texture = resources.get_texture(&self.texture_name)?;

        compute_pass.set_pipeline(material.get_pipeline());
        compute_pass.set_bind_group(0, texture.get_bind_group(), &[]);
        compute_pass.insert_debug_marker(&self.material_name);
        compute_pass.dispatch_workgroups(
            (texture.get_width() as f64 / 16.0).ceil() as u32,
            (texture.get_height() as f64 / 16.0).ceil() as u32,
            1,
        );

        Ok(())
    }
}