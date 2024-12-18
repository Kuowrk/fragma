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
mod compute_object;

pub use camera::Camera;
use scene::Scene;
use viewport::Viewport;
use resources::Resources;
use crate::renderer::resources::shader_data::ShaderPushConstants;

pub struct Renderer<'window> {
    viewport: Viewport<'window>,
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
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
            backends: wgpu::Backends::BROWSER_WEBGPU,
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
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits {
                            max_texture_dimension_1d: 8192,
                            max_texture_dimension_2d: 8192,
                            max_texture_dimension_3d: 2048,
                            max_texture_array_layers: 256,
                            max_bind_groups: 4,
                            max_buffer_size: 256 * 1024 * 1024, // 256 MB
                            max_vertex_buffers: 8,
                            max_vertex_attributes: 16,
                            max_vertex_buffer_array_stride: 2048,
                            max_push_constant_size: 128, // Typical browser support
                            max_storage_buffer_binding_size: 128 * 1024 * 1024, // 128 MB
                            ..Default::default()
                        }
                    } else {
                        wgpu::Limits {
                            max_push_constant_size: size_of::<ShaderPushConstants>() as u32,
                            ..Default::default()
                        }
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
        let resources = Rc::new(RefCell::new(Resources::new(&device, &queue, &viewport).await?));

        Ok(Self {
            viewport,
            device: Rc::new(device),
            queue: Rc::new(queue),
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

    pub fn resize(
        &mut self,
        new_size: PhysicalSize<u32>,
    ) {
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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            for compute_object in scene.get_compute_objects() {
                compute_object.dispatch(
                    &mut compute_pass,
                    &self.resources.borrow(),
                )?;
            }
        }

        let compute_texture = scene.get_compute_objects()[0].get_output_texture().unwrap();
        let copy_size = wgpu::Extent3d {
            width: output.texture.width().min(compute_texture.get_width()),
            height: output.texture.height().min(compute_texture.get_height()),
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: scene.get_compute_objects()[0].get_output_texture().unwrap().get_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &output.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            copy_size,
        );

        {
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        //load: wgpu::LoadOp::Clear(self.viewport.get_background()),
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            // Set push constants
            let push_constants = ShaderPushConstants {
                flipv: 1,
                gamma_correct: if self.viewport.get_surface_format().is_srgb() { 0 } else { 1 },
            };
            for render_object in scene.get_render_objects() {
                render_object.draw(
                    &mut render_pass,
                    camera,
                    &self.resources.borrow(),
                    &self.viewport,
                    &self.device,
                    &self.queue,
                    Some(&push_constants),
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
        Scene::new(
            self.device.clone(),
            self.queue.clone(),
            self.resources.clone(),
        )
    }

    pub fn set_vsync(&mut self, enable: bool) {
        self.viewport.set_vsync(enable);
    }
}