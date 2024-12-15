use color_eyre::Result;
use std::path::Path;

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

    pub fn new_from_file(filename: &str, device: &wgpu::Device) -> Result<Self> {
        let filepath = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("shaders")
            .join(filename);
        let source = std::fs::read_to_string(filepath)?;
        let desc = wgpu::ShaderModuleDescriptor {
            label: Some(filename),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        };
        Ok(Self::new_from_descriptor(desc, device))
    }

    pub fn get_module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}