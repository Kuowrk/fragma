use std::marker::PhantomData;
use super::resources::shader_data::{ShaderCameraUniform, ShaderSceneUniform};
use color_eyre::eyre::Result;

// Contains everything needed to draw a frame
pub struct FrameDrawContext<'a> {
    phantom_data: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct Frame {
    uniform_buffer: wgpu::Buffer,
}

impl Frame {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: (size_of::<ShaderSceneUniform>() + size_of::<ShaderCameraUniform>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            uniform_buffer,
        }
    }

    pub fn draw(&self, ctx: FrameDrawContext) -> Result<()> {
        Ok(())
    }
}