use wgpu::util::DeviceExt;
use color_eyre::eyre::{eyre, Result};
use wgpu::naga::back::spv::Capability::Shader;
use super::mesh::Mesh;
use super::shader_data::ShaderVertex;
use super::vertex::Vertex;
use super::super::viewport::Viewport;

pub struct FullscreenQuad {
    model: Model,
    width: f32,
    height: f32,
}

impl FullscreenQuad {
    pub fn new(viewport: &Viewport, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        let quad_mesh = Mesh::new_quad();
        let model = Model::new(vec![quad_mesh], device)?;
        let result = Self {
            model,
            width: 1.0,
            height: 1.0,
        };
        result.resize_to_viewport(viewport, device, queue);
        Ok(result)
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        self.model.draw(render_pass);
    }

    pub fn resize_to_viewport(
        &self,
        viewport: &Viewport,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        // Correct for image dimensions
        let mut x = if self.width >= self.height {
            1.0
        } else {
            self.width / self.height
        };
        let mut y = if self.width < self.height {
            1.0
        } else {
            self.height / self.width
        };

        // Correct for viewport dimensions
        let vp_size = viewport.get_size();
        if vp_size.width >= vp_size.height {
            y *= vp_size.width as f32 / vp_size.height as f32;
        } else {
            x *= vp_size.height as f32 / vp_size.width as f32;
        };

        // Update the background quad vertex buffer to match aspect ratio of background image
        let vertices_merged  = self.model
            .get_vertices_merged()
            .iter()
            .map(|v| {
                let p = v.position;
                let mut result = v.as_shader_data();
                result.position = glam::Vec3::new(p[0] * x, p[1] * y, p[2]);
                result
            })
            .collect::<Vec<ShaderVertex>>();
        let staging_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fullscreen Quad Vertex Staging Buffer"),
                contents: bytemuck::cast_slice(&vertices_merged),
                usage: wgpu::BufferUsages::COPY_SRC,
            });
        let mut encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Fullscreen Quad Vertex Buffer Update Encoder"),
            });
        let copy_size = size_of::<ShaderVertex>() * vertices_merged.len();
        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            self.model.get_vertex_buffer(),
            0,
            copy_size as wgpu::BufferAddress,
        );
        queue.submit(Some(encoder.finish()));
    }
}

#[derive(Debug)]
pub struct Model {
    meshes: Vec<Mesh>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: Option<wgpu::Buffer>,
}

impl Model {
    pub fn new(meshes: Vec<Mesh>, device: &wgpu::Device) -> Result<Self> {
        if meshes.is_empty() {
            return Err(eyre!("Model must have at least one mesh"));
        }

        // Ensure that all meshes have either no indices or all indices
        let has_indices = meshes.first().unwrap().indices.is_some();
        let all_meshes_valid = if has_indices {
            meshes.iter().all(|m| m.indices.is_some())
        } else {
            meshes.iter().all(|m| m.indices.is_none())
        };
        if !all_meshes_valid {
            return Err(eyre!("All meshes must have either no indices or all indices"));
        }

        // Collect all vertices from all meshes
        let vertices = meshes
            .iter()
            .flat_map(|m| m.vertices.iter())
            .map(|v| v.as_shader_data())
            .collect::<Vec<ShaderVertex>>();

        // Create a GPU-side vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Create a GPU-side index buffer if the model has indices
        let index_buffer = if has_indices {
            // Collect all indices from all meshes
            let indices = meshes
                .iter()
                .flat_map(|m| m.indices.as_ref().unwrap().iter().cloned())
                .collect::<Vec<u32>>();

            Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }))
        } else {
            None
        };


        Ok(Self {
            meshes,
            vertex_buffer,
            index_buffer,
        })
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        if let Some(index_buffer) = self.index_buffer.as_ref() {
            let index_count = self.meshes
                .iter()
                .map(|m| m.indices.as_ref().unwrap().len() as u32)
                .sum();
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..index_count, 0, 0..1);
        } else {
            let vertex_count = self.meshes
                .iter()
                .map(|m| m.vertices.len() as u32)
                .sum();
            render_pass.draw(0..vertex_count, 0..1);
        }
    }

    pub fn get_vertices_merged(&self) -> Vec<&Vertex> {
        self.meshes
            .iter()
            .flat_map(|m| m.vertices.iter())
            .collect()
    }

    pub fn get_indices_merged(&self) -> Option<Vec<&u32>> {
        if self.index_buffer.is_some() {
            Some(self.meshes
                .iter()
                .flat_map(|m| m.indices.as_ref().unwrap().iter())
                .collect())
        } else {
            None
        }
    }

    pub fn get_meshes(&self) -> &Vec<Mesh> {
        &self.meshes
    }

    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn get_index_buffer(&self) -> Option<&wgpu::Buffer> {
        self.index_buffer.as_ref()
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.meshes
            .iter()
            .zip(other.meshes.iter())
            .all(|(self_mesh, other_mesh)| self_mesh == other_mesh)
    }
}