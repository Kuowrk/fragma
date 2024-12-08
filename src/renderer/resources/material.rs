use color_eyre::eyre::{OptionExt, Result};

use super::shader::Shader;
use super::shader_data::{ShaderPushConstants, ShaderVertex};
use super::super::viewport::Viewport;

pub struct Material {
    pipeline: wgpu::RenderPipeline,
}

impl Material {
    pub fn builder<'a>() -> MaterialBuilder<'a> {
        MaterialBuilder::new()
    }

    pub fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

pub struct MaterialBuilder<'a> {
    shader: Option<Shader>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
}

impl<'a> MaterialBuilder<'a> {
    pub fn new() -> Self {
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

    pub fn build(mut self, device: &wgpu::Device, viewport: &Viewport) -> Result<Material> {
        let shader = self.shader.take().ok_or_eyre("No shader provided")?;
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &self.bind_group_layouts,
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..size_of::<ShaderPushConstants>() as u32,
                }],
            });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.get_module(),
                entry_point: Some("vs_main"),
                buffers: &[ShaderVertex::BUFFER_LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.get_module(),
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: viewport.get_config().format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                //cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        Ok(Material {
            pipeline,
        })
    }
}