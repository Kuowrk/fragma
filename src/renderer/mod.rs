use std::cell::RefCell;
use std::rc::Rc;
use color_eyre::eyre::{eyre, OptionExt, Result};
use winit::{dpi::PhysicalSize, window::Window};

pub mod viewport;
pub mod utils;
pub mod scene;
mod resources;
//mod frame;
mod camera;
mod render_object;

pub use camera::Camera;
use scene::Scene;
use viewport::Viewport;
use resources::Resources;

pub struct Renderer<'window> {
    viewport: Viewport<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    resources: Rc<RefCell<Resources>>,
}

impl<'window> Renderer<'window> {
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
        let resources = Rc::new(RefCell::new(Resources::new(&device, &queue, &viewport)?));

        Ok(Self {
            viewport,
            device,
            queue,
            resources,
        })
    }

    pub fn get_viewport(&self) -> &Viewport<'window> {
        &self.viewport
    }

    pub fn get_viewport_mut(&mut self) -> &mut Viewport<'window> {
        &mut self.viewport
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
            .borrow_mut()
            .get_fullscreen_quad_mut()
            .resize_to_viewport(&self.viewport, &self.device, &self.queue);
    }

    pub fn render(&mut self, camera: &mut Camera, scene: &Scene) -> Result<()> {
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

            for render_object in scene.get_render_objects() {
                render_object.draw(
                    &mut render_pass,
                    camera,
                    &self.resources.borrow(),
                    &self.viewport,
                    &self.device,
                    &self.queue,
                )?;
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn create_camera(&self) -> Camera {
        Camera::new(&self.device, &self.resources.borrow())
    }

    pub fn create_scene(&self) -> Scene {
        Scene::new(self.resources.clone())
    }

    pub fn set_vsync(&mut self, enable: bool) {
        self.viewport.set_vsync(enable);
    }
}