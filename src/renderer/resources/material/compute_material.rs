use color_eyre::eyre::OptionExt;
use color_eyre::Result;
use crate::renderer::resources::shader::Shader;
use crate::renderer::resources::shader_data::ShaderPushConstants;

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

    pub fn with_bind_group_layouts(mut self, bind_group_layouts: &[&'a wgpu::BindGroupLayout]) -> Self {
        self.bind_group_layouts = bind_group_layouts.into();
        self
    }

    pub fn build(mut self, device: &wgpu::Device) -> Result<ComputeMaterial> {
        let shader = self.shader.take().ok_or_eyre("No shader provided")?;
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &self.bind_group_layouts,
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: 0..size_of::<ShaderPushConstants>() as u32,
                }],
            });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader.get_module(),
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(ComputeMaterial {
            pipeline
        })
    }
}
