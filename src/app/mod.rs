mod camera_controller;

use color_eyre::eyre::Result;
use glam::Vec2;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};
use winit::dpi::PhysicalPosition;
use winit::error::ExternalError;
use winit::window::CursorGrabMode;
use crate::app::camera_controller::CameraController;
use crate::renderer::{Camera, Renderer};

#[derive(Default)]
struct InputState {
    mouse_curr_pos: Vec2,
    mouse_prev_pos: Vec2,
    mouse_wheel_delta_y: f32,
    mouse_left_down: bool,
    mouse_right_down: bool,
    mouse_right_down_pos: Vec2, // Position of the mouse when the right mouse button was pressed
}

pub struct App {
    event_loop: EventLoop<()>,
    window: Window,
}

impl App {
    pub fn new() -> Result<Self> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let window_size = winit::dpi::PhysicalSize::new(800, 600);
        let window = WindowBuilder::new()
            .with_title("Press R to toggle redraw requests")
            .with_inner_size(window_size)
            .with_resizable(true)
            .build(&event_loop)?;

        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            let _ = window.request_inner_size(window_size);

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("canvas-container")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    canvas.set_attribute("width", &window_size.width.to_string()).ok()?;
                    canvas.set_attribute("height", &window_size.height.to_string()).ok()?;
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Failed to append canvas to element with id=\"canvas-container\"");
        }

        Ok(Self {
            event_loop,
            window,
        })
    }

    pub async fn run(self) -> Result<()> {
        let mut renderer = Renderer::new(&self.window).await?;
        let mut camera_ctrl = CameraController::new(renderer.create_camera());
        let mut input_state = InputState::default();

        let mut request_redraws = true;
        let mut close_requested = false;

        self.event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    window_id,
                    ref event,
                } if window_id == renderer.get_window().id() => match event {
                    WindowEvent::CloseRequested => {
                        close_requested = true;
                    }
                    WindowEvent::RedrawRequested => {
                        renderer.get_window().pre_present_notify();
                        match renderer.render(camera_ctrl.get_camera_mut()) {
                            Ok(_) => {}
                            Err(report) => {
                                log::error!("{report}");
                                close_requested = true;
                            },
                        };
                    }
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        let mut new_size = renderer.get_viewport_size();
                        new_size.width = (new_size.width as f64 * scale_factor) as u32;
                        new_size.height = (new_size.height as f64 * scale_factor) as u32;

                        renderer.resize(new_size);
                    }
                    WindowEvent::KeyboardInput {
                        event:
                        KeyEvent {
                            logical_key: key,
                            state: ElementState::Pressed,
                            ..
                        },
                        ..
                    } => match key.as_ref() {
                        Key::Character("r") => {
                            request_redraws = !request_redraws;
                            log::info!("request_redraws: {}", request_redraws);
                        }
                        Key::Named(NamedKey::Escape) => {
                            close_requested = true;
                        }
                        _ => {}
                    },
                    WindowEvent::MouseInput {
                        state,
                        button: MouseButton::Right,
                        ..
                    } => {
                        match state {
                            ElementState::Pressed => {
                                renderer.get_window().set_cursor_grab(CursorGrabMode::Confined)
                                    .or_else(|_| renderer.get_window().set_cursor_grab(CursorGrabMode::Locked))
                                    .or_else(|e| {
                                        log::error!("Failed to grab cursor: {e}");
                                        Ok::<(), ExternalError>(())
                                    })
                                    .unwrap();
                                renderer.get_window().set_cursor_visible(false);

                                input_state.mouse_right_down = true;
                                input_state.mouse_right_down_pos = input_state.mouse_curr_pos;
                            }
                            ElementState::Released => {
                                renderer.get_window().set_cursor_grab(CursorGrabMode::None)
                                    .or_else(|e| {
                                        log::error!("Failed to release cursor: {e}");
                                        Ok::<(), ExternalError>(())
                                    })
                                    .unwrap();
                                renderer.get_window().set_cursor_visible(true);

                                input_state.mouse_right_down = false;
                                renderer
                                    .get_window()
                                    .set_cursor_position(PhysicalPosition::new(
                                        input_state.mouse_right_down_pos.x as f64,
                                        input_state.mouse_right_down_pos.y as f64
                                    ))
                                    .or_else(|e| {
                                        log::error!("Failed to set cursor position: {e}");
                                        Ok::<(), ExternalError>(())
                                    })
                                    .unwrap();
                            }
                        }
                    }
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => {
                        input_state.mouse_curr_pos = Vec2::new(position.x as f32, position.y as f32);
                        if input_state.mouse_right_down {
                            let viewport_size = renderer.get_viewport_size();
                            camera_ctrl.mouse_rotate(
                                input_state.mouse_prev_pos,
                                input_state.mouse_curr_pos,
                                viewport_size.width as f32,
                                viewport_size.height as f32,
                            );
                        }
                        input_state.mouse_prev_pos = input_state.mouse_curr_pos;
                    }
                    WindowEvent::MouseWheel {
                        delta,
                        ..
                    } => {
                        match delta {
                            MouseScrollDelta::LineDelta(_x, y) => {
                                input_state.mouse_wheel_delta_y = *y;
                            }
                            MouseScrollDelta::PixelDelta(pos) => {
                                input_state.mouse_wheel_delta_y = pos.y as f32;
                            }
                        }
                        camera_ctrl.mouse_zoom(input_state.mouse_wheel_delta_y);
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    if request_redraws {
                        renderer.get_window().request_redraw();
                    }

                    if close_requested {
                        elwt.exit();
                    }
                }
                _ => {}
            };
        })?;

        Ok(())
    }
}
