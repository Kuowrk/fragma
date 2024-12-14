use std::f32::consts::PI;
use glam::{Mat4, Vec2, Vec4, Vec4Swizzles};
use winit::dpi::PhysicalPosition;
use winit::error::ExternalError;
use winit::window::CursorGrabMode;
use crate::app::InputState;
use crate::renderer::Camera;
use crate::renderer::viewport::Viewport;

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

    pub fn process_input(
        &mut self,
        input_state: &mut InputState,
        viewport: &Viewport,
        delta_time: f32,
    ) {
        let viewport_center = Vec2::new(
            viewport.get_size().width as f32 / 2.0,
            viewport.get_size().height as f32 / 2.0,
        );

        if input_state.mouse_right_just_pressed {
            viewport.get_window().set_cursor_visible(false);
            self.set_mouse_pos(input_state, viewport, viewport_center);
        }
        else if input_state.mouse_right_just_released {
            viewport.get_window().set_cursor_visible(true);

            // Reset the cursor position to the position where the right mouse button was pressed
            self.set_mouse_pos(input_state, viewport, input_state.mouse_right_just_pressed_pos);
        }

        if input_state.mouse_right_down {
            let viewport_size = viewport.get_size();
            self.mouse_rotate(
                input_state.mouse_prev_pos,
                input_state.mouse_curr_pos,
                viewport_size.width as f32,
                viewport_size.height as f32,
                delta_time,
            );

            if self.mouse_just_left_border(input_state, viewport, 100) {
                self.set_mouse_pos(input_state, viewport, viewport_center);
            }
        }

        if input_state.mouse_wheel_delta_y != 0.0 {
            self.mouse_zoom(input_state.mouse_wheel_delta_y);
        }
    }

    fn mouse_zoom(&mut self, mouse_wheel_delta_y: f32) {
        let mut cam = &mut self.camera;
        let new_pos = cam.get_position() + cam.get_forward() * mouse_wheel_delta_y;
        if (new_pos - cam.get_pivot()).length() > cam.get_near() {
            cam.set_position(new_pos);
        }
    }

    fn mouse_rotate(
        &mut self,
        prev_mouse_pos: Vec2,
        curr_mouse_pos: Vec2,
        viewport_width: f32,
        viewport_height: f32,
        delta_time: f32,
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

    fn set_mouse_pos(
        &mut self,
        input_state: &mut InputState,
        viewport: &Viewport,
        pos: Vec2,
    ) {
        viewport.get_window().set_cursor_position(PhysicalPosition::new(
            pos.x as f64,
            pos.y as f64,
        ))
            .or_else(|e| {
                log::error!("Failed to set cursor position: {e}");
                Ok::<(), ExternalError>(())
            })
            .unwrap();
        input_state.mouse_curr_pos = pos;
        input_state.mouse_prev_pos = pos;
    }

    fn mouse_just_left_border(&self, input_state: &InputState, viewport: &Viewport, border_px: u32) -> bool {
        let pos = input_state.mouse_curr_pos;
        pos.x < border_px as f32
            || pos.y < border_px as f32
            || pos.x > viewport.get_size().width as f32 - border_px as f32
            || pos.y > viewport.get_size().height as f32 - border_px as f32
    }

}