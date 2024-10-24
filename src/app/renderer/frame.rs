use super::resources::shader_data::{ShaderCameraUniform, ShaderSceneUniform};
use super::context::Context;

// Contains everything needed to draw a frame
pub struct FrameDrawContext<'a> {

}

#[derive(Debug)]
pub struct Frame {
    uniform_buffer: wgpu::Buffer,
}

impl Frame {
    pub fn new(ctx: &Context) -> Self {
        let uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
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

    }
}