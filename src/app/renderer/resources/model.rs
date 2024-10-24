use wgpu::util::DeviceExt;
use crate::app::renderer::context::Context;
use crate::app::renderer::resources::mesh::Mesh;
use crate::app::renderer::resources::shader_data::ShaderVertex;
use color_eyre::eyre::{eyre, Result};

#[derive(Debug)]
pub struct Model {
    meshes: Vec<Mesh>,
    // Buffers start as None and are created when the model is loaded
    vertex_buffer: wgpu::Buffer,
    index_buffer: Option<wgpu::Buffer>,
}

impl Model {
    pub fn new(meshes: Vec<Mesh>, ctx: &Context) -> Result<Self> {
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
        let vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create a GPU-side index buffer if the model has indices
        let index_buffer = if has_indices {
            // Collect all indices from all meshes
            let indices = meshes
                .iter()
                .flat_map(|m| m.indices.as_ref().unwrap().iter())
                .collect::<Vec<u32>>();

            Some(ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

    pub fn get_meshes(&self) -> &Vec<Mesh> {
        &self.meshes
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