use std::f32::consts::PI;

use glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use wgpu::util::DeviceExt;
use crate::renderer::resources::Resources;
use crate::renderer::viewport::Viewport;
use super::resources::shader_data::ShaderCameraUniform;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);


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
}

impl Camera {
    const DEFAULT_FOV_Y_DEG: f32 = 60.0;

    pub fn new(
        device: &wgpu::Device,
        resources: &Resources,
        viewport: &Viewport
    ) -> Self {
        let camera_uniform_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Camera Uniform Buffer"),
                size: size_of::<ShaderCameraUniform>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
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

        let camera = Self {
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
        };

        // Populate the camera uniform buffer with the initial camera uniform data
        let vp_size = viewport.get_size();
        let camera_uniform_data = ShaderCameraUniform {
            viewproj: camera.get_viewproj_mat(vp_size.width as f32, vp_size.height as f32),
            near: camera.near,
            far: camera.far,
            _padding: [0.0; 2],
        };
        camera.camera_uniform_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(&[camera_uniform_data]));
        camera.camera_uniform_buffer.unmap();

        camera
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
    }

    pub fn mouse_zoom(&mut self, mouse_wheel_delta_y: f32) {
        let new_pos = self.position + self.forward * mouse_wheel_delta_y;
        if (new_pos - self.pivot).length() > self.near {
            self.position = new_pos;
        }
    }

    pub fn mouse_rotate(
        &mut self,
        prev_mouse_pos: Vec2,
        curr_mouse_pos: Vec2,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        // Get the homogeneous positions of the camera eye and pivot
        let pos =
            Vec4::new(self.position.x, self.position.y, self.position.z, 1.0);
        let piv = Vec4::new(self.pivot.x, self.pivot.y, self.pivot.z, 1.0);

        // Calculate the amount of rotation given the mouse movement
        let delta_angle_x = 2.0 * PI / viewport_width; // Left to right = 2*PI = 360deg
        let delta_angle_y = PI / viewport_height; // Top to bottom = PI = 180deg
        let angle_x = (prev_mouse_pos.x - curr_mouse_pos.x) * delta_angle_x;
        let angle_y = (prev_mouse_pos.y - curr_mouse_pos.y) * delta_angle_y;

        // Rotate the camera around the pivot point on the up axis
        let rot_x = Mat4::from_axis_angle(self.up, angle_x);
        let pos = (rot_x * (pos - piv)) + piv;

        // Rotate the camera around the pivot point on the right axis
        let rot_y = Mat4::from_axis_angle(self.right, angle_y);
        let pos = (rot_y * (pos - piv)) + piv;

        self.set_position(pos.xyz());
    }

    pub fn get_viewproj_mat(
        &self,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Mat4 {
        OPENGL_TO_WGPU_MATRIX * self.get_proj_mat(viewport_width, viewport_height) * self.get_view_mat()
    }

    pub fn get_view_mat(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward, self.up)
    }

    pub fn get_proj_mat(&self, viewport_width: f32, viewport_height: f32) -> Mat4 {
        let mut proj = Mat4::perspective_rh(
            self.fov_y_deg.to_radians(),
            viewport_width / viewport_height,
            self.near,
            self.far,
        );
        proj.y_axis.y *= -1.0;
        proj
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn get_near(&self) -> f32 {
        self.near
    }

    pub fn get_far(&self) -> f32 {
        self.far
    }

    pub fn get_bind_group_layout<'a>(&self, resources: &'a Resources) -> &'a wgpu::BindGroupLayout {
        resources.get_bind_group_layout("camera").unwrap()
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }
}