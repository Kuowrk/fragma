mod vertex;
mod viewport;
mod assets;

use color_eyre::eyre::{OptionExt, Result};
use image::{Rgba, RgbaImage};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use self::{vertex::Vertex, viewport::Viewport};

const FULLSCREEN_QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        uv: [1.0, 1.0],
    },
];

pub struct Renderer<'window> {
    viewport: Viewport<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    loaded_assets: assets::Assets,

    bg_quad_vbuffer: wgpu::Buffer,
    bg_bind_group: wgpu::BindGroup,
    bg_texture_height: u32,
    bg_texture_width: u32,
}

impl<'window> Renderer<'window> {
    /// Create a new renderer with all the state required to call `render_scene()`
    pub async fn new(window: &'window Window) -> Result<Renderer<'window>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::util::backend_bits_from_env()
                .unwrap_or(wgpu::Backends::PRIMARY),
            #[cfg(target_arch = "wasm32")]
            // NOTE: WebGPU is supported, but does not yet work in release version of Firefox
            //backends: wgpu::Backends::BROWSER_WEBGPU,
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_eyre("Failed to find an appropriate adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // Disable some features to support web
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let background = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        let viewport = Viewport::new(window, background, surface, &adapter)?;

        let shader = device.create_shader_module(wgpu::include_wgsl!("../../../shaders/basic.wgsl"));

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let bg_quad_vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Background Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(FULLSCREEN_QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
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
                cull_mode: Some(wgpu::Face::Back),
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

        let loaded_assets = {
            let mut assets = assets::Assets::default();

            let mut black_image = RgbaImage::new(1, 1);
            black_image.put_pixel(0, 0, Rgba([0, 0, 0, 255]));
            assets.add_image("Black".to_owned(), black_image);

            let mut white_image = RgbaImage::new(1, 1);
            white_image.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
            assets.add_image("White".to_owned(), white_image);

            assets
        };

        let bg_bind_group = Self::create_texture(
            loaded_assets.get_image("Black").ok_or_eyre("Failed to load black image")?,
            "background_texture",
            &device,
            &queue,
            &texture_bind_group_layout,
        );

        Ok(Self {
            viewport,
            device,
            queue,
            render_pipeline,
            texture_bind_group_layout,
            loaded_assets,
            bg_quad_vbuffer,
            bg_bind_group,
            bg_texture_width: 1,
            bg_texture_height: 1,
        })
    }

    pub fn get_size(&self) -> PhysicalSize<u32> {
        self.viewport.get_size()
    }

    pub fn get_window(&self) -> &Window {
        self.viewport.get_window()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.viewport.resize(new_size, &self.device);
        self.resize_background_texture();
    }

    pub fn update_scene(&mut self) {}

    pub fn render_scene(&mut self) -> core::result::Result<(), wgpu::SurfaceError> {
        let output = self.viewport.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.viewport.get_background()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bg_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.bg_quad_vbuffer.slice(..));
            render_pass.draw(0..FULLSCREEN_QUAD_VERTICES.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn set_background_image(&mut self, image: &RgbaImage) {
        self.bg_texture_width = image.width();
        self.bg_texture_height = image.height();
        self.bg_bind_group = Self::create_texture(
            image,
            "background_texture",
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
        );

        self.resize_background_texture();
    }

    fn resize_background_texture(&self) {
        // Update the background quad vertex buffer to match aspect ratio of background image
        // Correct for image dimensions
        let image_width = self.bg_texture_width as f32;
        let image_height = self.bg_texture_height as f32;
        let mut x = if image_width >= image_height {
            1.0
        } else {
            image_width / image_height
        };
        let mut y = if image_width < image_height {
            1.0
        } else {
            image_height / image_width
        };
        // Correct for viewport dimensions
        let vp_size = self.viewport.get_size();
        if vp_size.width >= vp_size.height {
            y *= vp_size.width as f32 / vp_size.height as f32;
        } else {
            x *= vp_size.height as f32 / vp_size.width as f32;
        };
        let vertices: Vec<Vertex> = FULLSCREEN_QUAD_VERTICES
            .iter()
            .map(|v| {
                let p = v.position;
                Vertex {
                    position: [p[0] * x, p[1] * y, p[2]],
                    uv: v.uv,
                }
            })
            .collect();
        let staging_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("staging_buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::COPY_SRC,
            });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Background Quad Vertex Buffer Update Encoder"),
            });
        let copy_size = std::mem::size_of::<Vertex>() * vertices.len();
        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.bg_quad_vbuffer,
            0,
            copy_size as wgpu::BufferAddress,
        );
        self.queue.submit(Some(encoder.finish()));
    }

    fn create_texture(
        image: &RgbaImage,
        texture_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let dimensions = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(texture_name),
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some(&format!("{}_bind_group", texture_name)),
        })
    }
}