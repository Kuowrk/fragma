use super::resources::shader::Shader;
use super::viewport::Viewport;
use color_eyre::eyre::{OptionExt, Result};
use image::{DynamicImage, GenericImage, ImageBuffer};
use std::collections::HashMap;
use wgpu::include_wgsl;
use crate::renderer::resources::model::FullscreenQuad;

pub mod mesh;
pub mod vertex;
pub mod shader;
pub mod model;
mod material;
mod texture;
pub mod shader_data;

/// Global resources
pub struct Resources {
    models: HashMap<String, model::Model>,
    textures: HashMap<String, texture::Texture>,
    materials: HashMap<String, material::Material>,
    fullscreen_quad: FullscreenQuad,

    // wgpu resources
    samplers : HashMap<String, wgpu::Sampler>,
    bind_group_layouts: HashMap<String, wgpu::BindGroupLayout>,
}

impl Resources {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &Viewport) -> Result<Self> {
        let bind_group_layouts = create_default_bind_group_layouts(device);
        let samplers = create_default_samplers(device)?;
        let materials = create_default_materials(&bind_group_layouts, device, viewport)?;
        let models = create_default_models(device)?;
        let fullscreen_quad = FullscreenQuad::new(viewport, device, queue)?;
        let mut result = Self {
            models,
            textures: HashMap::new(),
            materials,
            samplers,
            bind_group_layouts,
            fullscreen_quad,
        };
        // Default textures depends on the bind group layouts and samplers
        result.textures = create_default_textures(device, queue, &result)?;
        Ok(result)
    }

    pub fn get_model(&self, name: &str) -> Result<&model::Model> {
        self.models.get(name).ok_or_eyre(format!("Failed to get model: {name}"))
    }

    pub fn get_texture(&self, name: &str) -> Result<&texture::Texture> {
        self.textures.get(name).ok_or_eyre(format!("Failed to get texture: {name}"))
    }

    pub fn get_material(&self, name: &str) -> Result<&material::Material> {
        self.materials.get(name).ok_or_eyre(format!("Failed to get material: {name}"))
    }

    pub fn get_sampler(&self, name: &str) -> Result<&wgpu::Sampler> {
        self.samplers.get(name).ok_or_eyre(format!("Failed to get sampler: {name}"))
    }

    pub fn get_bind_group_layout(&self, name: &str) -> Result<&wgpu::BindGroupLayout> {
        self.bind_group_layouts.get(name).ok_or_eyre(format!("Failed to get bind group layout: {name}"))
    }

    pub fn get_fullscreen_quad(&self) -> &FullscreenQuad {
        &self.fullscreen_quad
    }

    pub fn get_fullscreen_quad_mut(&mut self) -> &mut FullscreenQuad {
        &mut self.fullscreen_quad
    }
}

fn create_default_models(
    device: &wgpu::Device,
) -> Result<HashMap<String, model::Model>> {
    let mut result = HashMap::new();

    result.insert("triangle".to_owned(), model::Model::new(
        vec![mesh::Mesh::new_triangle()],
        device,
    )?);
    result.insert("quad".to_owned(), model::Model::new(
        vec![mesh::Mesh::new_quad()],
        device,
    )?);

    Ok(result)
}

fn create_default_textures(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resources: &Resources,
) -> Result<HashMap<String, texture::Texture>> {
    let mut result = HashMap::new();

    let mut black_image = image::RgbaImage::new(1, 1);
    black_image.put_pixel(0, 0, image::Rgba([0, 0, 0, 255]));
    result.insert("black".to_owned(), texture::Texture::new_from_image(
        &black_image.into(),
        "black",
        device,
        queue,
        resources,
    )?);

    let mut white_image = image::RgbaImage::new(1, 1);
    white_image.put_pixel(0, 0, image::Rgba([255, 255, 255, 255]));
    result.insert("white".to_owned(), texture::Texture::new_from_image(
        &white_image.into(),
        "white",
        device,
        queue,
        resources,
    )?);

    result.insert("tree".to_owned(), texture::Texture::new_from_bytes(
        include_bytes!("../../../assets/tree.png"),
        "tree",
        device,
        queue,
        resources,
    )?);

    Ok(result)
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
        .with_shader(Shader::new_from_descriptor(include_wgsl!("../../../shaders/basic.wgsl"), device))
        .build(device, viewport)?);
    Ok(result)
}

fn create_default_samplers(
    device: &wgpu::Device,
) -> Result<HashMap<String, wgpu::Sampler>> {
    let mut result = HashMap::new();
    result.insert("nearest".to_owned(), device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        label: Some("Nearest Sampler"),
        ..Default::default()
    }));
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

