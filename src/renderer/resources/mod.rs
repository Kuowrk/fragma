pub mod mesh;
pub mod vertex;
pub mod shader;
pub mod model;
pub mod material;
pub mod texture;
pub mod shader_data;

use color_eyre::eyre::{OptionExt, Result, eyre};
use std::collections::HashMap;
use super::viewport::Viewport;
use shader::Shader;
use model::FullscreenQuad;
use crate::renderer::compute_object::ComputeObject;
use crate::renderer::render_object::RenderObject;
use crate::renderer::resources::material::compute_material::ComputeMaterial;
use crate::renderer::resources::material::render_material::RenderMaterial;

const SINGLE_TEXTURE_BIND_GROUP_LAYOUT_NAME: &str = "single texture";
const CAMERA_BIND_GROUP_LAYOUT_NAME: &str = "camera";
const COMPUTE_STORAGE_TEXTURE_NAME: &str = "compute storage";

/// Global resources
pub struct Resources {
    models: HashMap<String, model::Model>,
    textures: HashMap<String, texture::Texture>,
    render_materials: HashMap<String, RenderMaterial>,
    compute_materials: HashMap<String, ComputeMaterial>,
    fullscreen_quad: FullscreenQuad,

    // wgpu resources
    samplers : HashMap<String, wgpu::Sampler>,
    bind_group_layouts: HashMap<String, wgpu::BindGroupLayout>,
}

impl Resources {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: &Viewport,
    ) -> Result<Self> {
        let bind_group_layouts = create_default_bind_group_layouts(device);
        let samplers = create_default_samplers(device)?;
        let render_materials = create_default_render_materials(&bind_group_layouts, device, viewport)?;
        let compute_materials = create_default_compute_materials(&bind_group_layouts, device)?;
        let models = create_default_models(device)?;
        let fullscreen_quad = FullscreenQuad::new(viewport, device, queue)?;
        let mut result = Self {
            models,
            textures: HashMap::new(),
            render_materials,
            compute_materials,
            samplers,
            bind_group_layouts,
            fullscreen_quad,
        };
        // Default textures depends on the bind group layouts and samplers
        result.textures = create_default_textures(device, queue, &result)?;
        Ok(result)
    }

    pub fn create_render_object(
        &self,
        material_name: &str,
        texture_name: &str,
        model_name: &str,
    ) -> Result<RenderObject> {
        let material_exists = self.render_materials.contains_key(material_name);
        let texture_exists = self.textures.contains_key(texture_name);
        let model_exists = self.models.contains_key(model_name);

        if !material_exists {
            return Err(eyre!("Material not found: {}", material_name));
        }
        if !texture_exists {
            return Err(eyre!("Texture not found: {}", texture_name));
        }
        if !model_exists {
            return Err(eyre!("Model not found: {}", model_name));
        }

        Ok(RenderObject::new(
            material_name.to_owned(),
            texture_name.to_owned(),
            model_name.to_owned(),
        ))
    }

    pub fn create_compute_object(
        &self,
        material_name: &str,
    ) -> Result<ComputeObject> {
        let material_exists = self.compute_materials.contains_key(material_name);

        if !material_exists {
            return Err(eyre!("Material not found: {}", material_name));
        }

        Ok(ComputeObject::new(
            material_name.to_owned(),
            COMPUTE_STORAGE_TEXTURE_NAME.to_owned(),
        ))
    }

    pub fn get_model(&self, name: &str) -> Result<&model::Model> {
        self.models.get(name).ok_or_eyre(format!("Failed to get model: {name}"))
    }

    pub fn get_texture(&self, name: &str) -> Result<&texture::Texture> {
        self.textures.get(name).ok_or_eyre(format!("Failed to get texture: {name}"))
    }

    pub fn get_render_material(&self, name: &str) -> Result<&RenderMaterial> {
        self.render_materials.get(name).ok_or_eyre(format!("Failed to get render material: {name}"))
    }

    pub fn get_compute_material(&self, name: &str) -> Result<&ComputeMaterial> {
        self.compute_materials.get(name).ok_or_eyre(format!("Failed to get compute material: {name}"))
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

    result.insert(COMPUTE_STORAGE_TEXTURE_NAME.to_owned(), texture::Texture::new_compute_storage(
        COMPUTE_STORAGE_TEXTURE_NAME,
        1,
        1,
        device,
        resources,
    )?);

    Ok(result)
}

fn create_default_render_materials(
    bind_group_layouts: &HashMap<String, wgpu::BindGroupLayout>,
    device: &wgpu::Device,
    viewport: &Viewport,
) -> Result<HashMap<String, RenderMaterial>> {
    let mut result = HashMap::new();

    result.insert("basic".to_owned(), RenderMaterial::builder()
        .with_bind_group_layouts(&[
            bind_group_layouts.get(SINGLE_TEXTURE_BIND_GROUP_LAYOUT_NAME).unwrap(),
            bind_group_layouts.get(CAMERA_BIND_GROUP_LAYOUT_NAME).unwrap(),
        ])
        .with_shader(Shader::new_from_file("basic.wgsl", device)?)
        .build(device, viewport)?);

    Ok(result)
}

fn create_default_compute_materials(
    bind_group_layouts: &HashMap<String, wgpu::BindGroupLayout>,
    device: &wgpu::Device,
) -> Result<HashMap<String, ComputeMaterial>> {
    let mut result = HashMap::new();

    result.insert("basic compute".to_owned(), ComputeMaterial::builder()
        .with_bind_group_layouts(&[
            bind_group_layouts.get("compute storage").unwrap(),
        ])
        .with_shader(Shader::new_from_file("basic_compute.wgsl", device)?)
        .build(device)?);

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

    result.insert(SINGLE_TEXTURE_BIND_GROUP_LAYOUT_NAME.to_owned(), device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    result.insert(CAMERA_BIND_GROUP_LAYOUT_NAME.to_owned(), device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
        label: Some("Camera Bind Group Layout"),
    }));

    result.insert(COMPUTE_STORAGE_TEXTURE_NAME.to_owned(), device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }
        ],
        label: Some("Compute Storage Bind Group Layout"),
    }));

    result
}

