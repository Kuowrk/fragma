use color_eyre::eyre::OptionExt;
use color_eyre::Result;
use crate::renderer::resources::Resources;
use crate::renderer::resources::texture::Texture;

pub struct ComputeObject {
    compute_material_name: String,
    output_texture: Option<Texture>,
}

impl ComputeObject {
    pub fn new_with_output_texture(
        compute_material_name: String,
        texture_width: u32,
        texture_height: u32,
        device: &wgpu::Device,
        resources: &Resources,
    ) -> Result<Self> {
        let output_texture = Texture::new_compute_storage(
            "Compute Storage Texture",
            texture_width,
            texture_height,
            device,
            resources,
        )?;
        Ok(Self {
            compute_material_name,
            output_texture: Some(output_texture),
        })
    }

    pub fn resize_output_texture(
        &mut self,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        resources: &Resources,
    ) -> Result<()> {
        let old_output_texture = self.output_texture.take()
            .ok_or_eyre("No output texture")?;
        let new_output_texture = Texture::new_compute_storage(
            "Compute Storage Texture",
            width,
            height,
            device,
            resources,
        )?;

        let copy_width = old_output_texture.get_width().min(new_output_texture.get_width());
        let copy_height = old_output_texture.get_height().min(new_output_texture.get_height());
        let copy_size = wgpu::Extent3d {
            width: copy_width,
            height: copy_height,
            depth_or_array_layers: 1,
        };

        let mut encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: &old_output_texture.get_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: new_output_texture.get_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            copy_size,
        );
        queue.submit(std::iter::once(encoder.finish()));

        self.output_texture = Some(new_output_texture);

        Ok(())
    }

    pub fn dispatch(
        &self,
        compute_pass: &mut wgpu::ComputePass,
        resources: &Resources,
    ) -> Result<()> {
        let material = resources.get_compute_material(&self.compute_material_name)?;

        let mut work_groups_x = 16;
        let mut work_groups_y = 16;

        compute_pass.set_pipeline(material.get_pipeline());
        if let Some(texture) = &self.output_texture {
            compute_pass.set_bind_group(0, texture.get_bind_group(), &[]);
            compute_pass.insert_debug_marker(&self.compute_material_name);
            work_groups_x = (texture.get_width() as f64 / 16.0).ceil() as u32;
            work_groups_y = (texture.get_height() as f64 / 16.0).ceil() as u32;
        }

        compute_pass.dispatch_workgroups(
            work_groups_x,
            work_groups_y,
            1,
        );

        Ok(())
    }

    pub fn has_output_texture(&self) -> bool {
        self.output_texture.is_some()
    }

    pub fn get_output_texture(&self) -> Option<&Texture> {
        self.output_texture.as_ref()
    }
}