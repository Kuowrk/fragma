use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec2};

/* This module contains data to be sent to and from shaders. */

/// Scene-related data
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ShaderSceneUniform {
}

/// Camera-related data
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ShaderCameraUniform {
    pub viewproj: Mat4,
    pub near: f32,
    pub far: f32,
    pub _padding: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ShaderPushConstants {
}

/// Vertex data
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ShaderVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
    pub texcoord: Vec2,
}

impl ShaderVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![
            0 => Float32x3, // position
            1 => Float32x3, // normal
            2 => Float32x3, // color
            3 => Float32x2, // texcoord
        ];

    pub const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &Self::ATTRIBS,
    };
}
