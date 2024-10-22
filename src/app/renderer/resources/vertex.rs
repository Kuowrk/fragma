use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
    pub texcoord: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub const FULLSCREEN_QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0],
        texcoord: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0],
        texcoord: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0],
        texcoord: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0],
        texcoord: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0],
        texcoord: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        color: [1.0, 0.0, 1.0],
        texcoord: [1.0, 1.0],
    },
];
