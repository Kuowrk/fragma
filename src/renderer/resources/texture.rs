use color_eyre::eyre::{OptionExt, Result};
use crate::renderer::resources::vertex::Vertex;
use super::Resources;

#[derive(Debug)]
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn new_from_bytes(
        bytes: &[u8],
        label: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        resources: &Resources,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::new_from_image(&img, label, device, queue, resources)
    }

    pub fn new_from_image(
        image: &image::DynamicImage,
        label: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        resources: &Resources,
    ) -> Result<Self> {
        let image = image.to_rgba8();
        let dimensions = image.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(label),
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = resources.get_sampler("nearest")?;

        let layout = resources.get_bind_group_layout("single texture")?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some(&format!("{label} Bind Group")),
        });

        Ok(Self {
            texture,
            view,
            bind_group,
            width: size.width,
            height: size.height,
        })
    }


    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}