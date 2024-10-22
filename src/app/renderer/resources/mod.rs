pub mod mesh;
pub mod vertex;

// Resources are more system-focused compared to Assets, which are more content-focused.
// Resources are usually created in memory, while Assets are usually loaded from disk or network.
#[derive(Debug)]
pub struct Resources {
    single_texture_bind_group_layout: wgpu::BindGroupLayout,
}