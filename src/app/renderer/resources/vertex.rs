use glam::{Vec2, Vec3};
use crate::app::renderer::resources::shader_data::ShaderVertex;

#[derive(Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
    pub texcoord: Vec2,
}

impl Vertex {
    pub fn as_shader_data(&self) -> ShaderVertex {
        ShaderVertex {
            position: self.position.into(),
            normal: self.normal.into(),
            color: self.color.into(),
            texcoord: self.texcoord.into(),
        }
    }
}
