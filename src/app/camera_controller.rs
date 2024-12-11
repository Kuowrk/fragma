use std::f32::consts::PI;
use glam::{Mat4, Vec2, Vec4, Vec4Swizzles};
use crate::renderer::Camera;

pub struct CameraController {
    camera: Camera,
}

impl CameraController {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
        }
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn mouse_zoom(&mut self, mouse_wheel_delta_y: f32) {
        let mut cam = &mut self.camera;
        let new_pos = cam.get_position() + cam.get_forward() * mouse_wheel_delta_y;
        if (new_pos - cam.get_pivot()).length() > cam.get_near() {
            cam.set_position(new_pos);
        }
    }

    pub fn mouse_rotate(
        &mut self,
        prev_mouse_pos: Vec2,
        curr_mouse_pos: Vec2,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        let cam = &mut self.camera;
        let cam_pos = cam.get_position();
        let cam_piv = cam.get_pivot();

        // Get the homogeneous positions of the camera eye and pivot
        let pos = Vec4::new(cam_pos.x, cam_pos.y, cam_pos.z, 1.0);
        let piv = Vec4::new(cam_piv.x, cam_piv.y, cam_piv.z, 1.0);

        // Calculate the amount of rotation given the mouse movement
        let delta_angle_x = 2.0 * PI / viewport_width; // Left to right = 2*PI = 360deg
        let delta_angle_y = PI / viewport_height; // Top to bottom = PI = 180deg
        let angle_x = (prev_mouse_pos.x - curr_mouse_pos.x) * delta_angle_x;
        let angle_y = (prev_mouse_pos.y - curr_mouse_pos.y) * delta_angle_y;

        // Rotate the camera around the pivot point on the up axis
        let rot_x = Mat4::from_axis_angle(cam.get_up(), angle_x);
        let pos = (rot_x * (pos - piv)) + piv;

        // Rotate the camera around the pivot point on the right axis
        let rot_y = Mat4::from_axis_angle(cam.get_right(), angle_y);
        let pos = (rot_y * (pos - piv)) + piv;

        cam.set_position(pos.xyz());
    }

}