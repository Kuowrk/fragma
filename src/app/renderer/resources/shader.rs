#[derive(Debug)]
pub struct Shader {
    module: wgpu::ShaderModule,
}

impl Shader {
    pub fn new_from_descriptor(desc: wgpu::ShaderModuleDescriptor, device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(desc);
        Self {
            module
        }
    }

    pub fn get_module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}