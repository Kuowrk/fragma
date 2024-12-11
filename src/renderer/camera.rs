use std::f32::consts::PI;
use std::sync::Arc;
use glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use wgpu::util::DeviceExt;
use crate::renderer::resources::Resources;
use crate::renderer::viewport::Viewport;
use super::resources::shader_data::{ShaderCameraUniform, ShaderVertex};

pub struct Camera {
    position: Vec3,
    forward: Vec3,
    up: Vec3,
    right: Vec3,
    world_up: Vec3,
    fov_y_deg: f32,
    near: f32,
    far: f32,
    pivot: Vec3,

    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    dirty: bool,
}

impl Camera {
    const DEFAULT_FOV_Y_DEG: f32 = 45.0;

    pub fn new(
        device: &wgpu::Device,
        resources: &Resources,
    ) -> Self {
        let camera_uniform_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Camera Uniform Buffer"),
                size: size_of::<ShaderCameraUniform>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: resources.get_bind_group_layout("camera").unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("Camera Bind Group"),
        });

        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            forward: Vec3::NEG_Z,
            up: Vec3::Y,
            right: Vec3::X,
            world_up: Vec3::Y,
            fov_y_deg: Self::DEFAULT_FOV_Y_DEG,
            near: 0.1,
            far: 100.0,
            pivot: Vec3::ZERO,
            camera_uniform_buffer,
            camera_bind_group,
            dirty: true,
        }
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.look_at(self.pivot);
    }

    pub fn look_at(&mut self, target: Vec3) {
        if target == self.position {
            return;
        }
        self.pivot = target;
        self.forward = (target - self.position).normalize();
        self.right = self.forward.cross(self.world_up).normalize();
        self.up = self.right.cross(self.forward).normalize();
        self.dirty = true;
    }

    pub fn get_viewproj_mat(
        &self,
        viewport: &Viewport,
    ) -> Mat4 {
        self.get_proj_mat(viewport) * self.get_view_mat()
    }

    pub fn get_view_mat(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward, self.up)
    }

    pub fn get_proj_mat(
        &self,
        viewport: &Viewport,
    ) -> Mat4 {
        let vp_size = viewport.get_size();
        let aspect_ratio = vp_size.width as f32 / vp_size.height as f32;
        Mat4::perspective_rh(
            self.fov_y_deg.to_radians(),
            aspect_ratio,
            self.near,
            self.far,
        )
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn get_forward(&self) -> Vec3 {
        self.forward
    }

    pub fn get_up(&self) -> Vec3 {
        self.up
    }

    pub fn get_right(&self) -> Vec3 {
        self.right
    }

    pub fn get_near(&self) -> f32 {
        self.near
    }

    pub fn get_far(&self) -> f32 {
        self.far
    }

    pub fn get_pivot(&self) -> Vec3 {
        self.pivot
    }

    pub fn get_bind_group_layout<'a>(&self, resources: &'a Resources) -> &'a wgpu::BindGroupLayout {
        resources.get_bind_group_layout("camera").unwrap()
    }

    pub fn get_bind_group(
        &mut self,
        viewport: &Viewport,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &wgpu::BindGroup {
        if self.dirty {
            self.update_uniform_buffer(viewport, device, queue);
            self.dirty = false;
        }
        &self.camera_bind_group
    }

    fn update_uniform_buffer(
        &self,
        viewport: &Viewport,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let camera_uniform_data = ShaderCameraUniform {
            viewproj: self.get_viewproj_mat(viewport),
            near: self.near,
            far: self.far,
            _padding: [0.0; 2],
        };

        let staging_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Uniform Staging Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform_data]),
                usage: wgpu::BufferUsages::COPY_SRC,
            });
        let mut encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Camera Uniform Buffer Update Encoder"),
            });
        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.camera_uniform_buffer,
            0,
            size_of::<ShaderCameraUniform>() as wgpu::BufferAddress,
        );
        queue.submit(Some(encoder.finish()));
    }
}