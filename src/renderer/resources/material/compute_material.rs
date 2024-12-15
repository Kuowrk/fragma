use crate::renderer::resources::shader::Shader;

pub struct ComputeMaterial {
    pipeline: wgpu::ComputePipeline,
}

impl ComputeMaterial {
    pub fn builder<'a>() -> ComputeMaterialBuilder<'a> {
        ComputeMaterialBuilder::new()
    }

    pub fn get_pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}

pub struct ComputeMaterialBuilder<'a> {
    shader: Option<Shader>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
}

impl<'a> ComputeMaterialBuilder<'a> {
    fn new() -> Self {
        Self {
            shader: None,
            bind_group_layouts: Vec::new(),
        }
    }

    pub fn with_shader(mut self, shader: Shader) -> Self {
        self.shader = Some(shader);
        self
    }
}
