use crate::camera::CameraController;
use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::PhysicalKey,
    window::Window,
};

pub struct InputHandler {
    pub camera_controller: CameraController,
    pub right_mouse_pressed: bool,
    pub is_scene_focused: bool,
}

impl InputHandler {
    pub fn new(camera_speed: f32) -> Self {
        Self {
            camera_controller: CameraController::new(camera_speed),
            right_mouse_pressed: false,
            is_scene_focused: false,
        }
    }

    pub fn process_input(
        &mut self,
        event: &WindowEvent,
        window: &Window,
        egui_consumed: bool,
        is_scene_hovered: bool,
    ) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                if self.is_scene_focused && !egui_consumed {
                    self.camera_controller.process_keyboard(*keycode, *state)
                } else {
                    false
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                self.right_mouse_pressed = is_pressed;

                if is_pressed {
                    if is_scene_hovered {
                        self.is_scene_focused = true;
                        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
                        window.set_cursor_visible(false);
                        true
                    } else {
                        false
                    }
                } else {
                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                    window.set_cursor_visible(true);
                    true
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                if is_scene_hovered {
                    self.is_scene_focused = true;
                } else if egui_consumed {
                    self.is_scene_focused = false;
                }
                false
            }
            _ => false,
        }
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        if self.right_mouse_pressed {
            self.camera_controller.process_mouse(delta.0, delta.1);
        }
    }
}
