use std::collections::HashMap;
use wgpu::include_wgsl;
use super::viewport::Viewport;
use super::resources::shader::Shader;
use color_eyre::eyre::Result;

pub mod mesh;
pub mod vertex;
pub mod shader;
pub mod model;
mod material;
mod texture;
pub mod shader_data;

/// Global resources
pub struct Resources {
    pub models: HashMap<String, model::Model>,
    pub textures: HashMap<String, texture::Texture>,
    pub materials: HashMap<String, material::Material>,

    // wgpu resources
    pub samplers : HashMap<String, wgpu::Sampler>,
    pub bind_group_layouts: HashMap<String, wgpu::BindGroupLayout>,
}

impl Resources {
    fn new(device: &wgpu::Device, viewport: &Viewport) -> Result<Self> {
        let bind_group_layouts = create_default_bind_group_layouts(device);
        let materials = create_default_materials(&bind_group_layouts, device, viewport)?;
        Ok(Self {
            models: HashMap::new(),
            textures: HashMap::new(),
            materials,
            samplers: HashMap::new(),
            bind_group_layouts,
        })
    }
}

fn create_default_materials(
    bind_group_layouts: &HashMap<String, wgpu::BindGroupLayout>,
    device: &wgpu::Device,
    viewport: &Viewport
) -> Result<HashMap<String, material::Material>> {
    let mut result = HashMap::new();
    result.insert("basic".to_owned(), material::Material::builder()
        .with_bind_group_layouts(&[
            bind_group_layouts.get("single texture").unwrap(),
        ])
        .with_shader(Shader::new_from_descriptor(include_wgsl!("../../../shaders/basic.wgsl")))
        .build(device, viewport)?);
    Ok(result)
}

fn create_default_bind_group_layouts(device: &wgpu::Device) -> HashMap<String, wgpu::BindGroupLayout> {
    let mut result = HashMap::new();
    result.insert("single texture".to_owned(), device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("Single Texture Bind Group Layout"),
    }));
    result
}

