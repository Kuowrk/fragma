use color_eyre::eyre::{eyre, OptionExt, Result};
use image::{Rgba, RgbaImage};
use wgpu::SurfaceTexture;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

mod viewport;
mod resources;
mod frame;
mod camera;
mod scene;

pub use camera::Camera;
use scene::Scene;
use viewport::Viewport;
use resources::Resources;
use resources::vertex::Vertex;
use resources::model::FullscreenQuad;

pub struct Renderer<'window> {
    viewport: Viewport<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    resources: Resources,
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
                    required_features: wgpu::Features::PUSH_CONSTANTS,
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
            .map_err(|e| eyre!(e.to_string()))?;

        let background = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        let viewport = Viewport::new(window, background, surface, &adapter)?;
        let resources = Resources::new(&device, &queue, &viewport)?;

        Ok(Self {
            viewport,
            device,
            queue,
            resources,
        })
    }

    pub fn get_viewport_size(&self) -> PhysicalSize<u32> {
        self.viewport.get_size()
    }

    pub fn get_window(&self) -> &Window {
        self.viewport.get_window()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.viewport.resize(new_size, &self.device);
        self.resources
            .get_fullscreen_quad_mut()
            .resize_to_viewport(&self.viewport, &self.device, &self.queue);
    }

    pub fn render(&mut self, camera: &mut Camera) -> Result<()> {
        let output = match self.viewport.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Lost) => {
                self.resize(self.get_viewport_size());
                self.viewport.get_current_texture()?
            },
            Err(wgpu::SurfaceError::OutOfMemory) => {
                return Err(wgpu::SurfaceError::OutOfMemory.into());
            },
            Err(e) => {
                log::error!("Unexpected error: {:?}", e);
                return Ok(())
            },
        };

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

            /*
            let material = self.resources.get_material("basic")?;
            let texture = self.resources.get_texture("tree")?;
            let model = self.resources.get_fullscreen_quad();
            render_pass.set_pipeline(material.get_pipeline());
            render_pass.set_bind_group(0, texture.get_bind_group(), &[]);
            render_pass.set_bind_group(1, camera.get_bind_group(), &[]);
            model.draw(&mut render_pass);
            */

            let material = self.resources.get_material("basic")?;
            let texture = self.resources.get_texture("white")?;
            let model = self.resources.get_model("triangle")?;
            render_pass.set_pipeline(material.get_pipeline());
            render_pass.set_bind_group(0, texture.get_bind_group(), &[]);
            render_pass.set_bind_group(1, camera.get_bind_group(
                &self.viewport,
                &self.device,
                &self.queue,
            ), &[]);
            model.draw(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn create_camera(&self) -> Camera {
        Camera::new(&self.device, &self.resources)
    }
}