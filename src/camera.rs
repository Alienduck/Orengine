use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

// 1. Real camera (CPU side)
pub struct Camera {
    pub eye: glam::Vec3,    // Eye position
    pub target: glam::Vec3, // What we look at
    pub up: glam::Vec3,     // Up vector (Y)
    pub aspect: f32,        // Screen ratio
    pub fovy: f32,          // Vertical field of view
    pub znear: f32,         // Minimum display distance
    pub zfar: f32,          // Maximum display distance
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

// 2. Struct of the uniform buffer (GPU side)
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use glam::Mat4;
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

// 3. Controller to handle input and update the camera
pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_right_mouse_pressed: bool,
    last_mouse_pos: (f32, f32),
    mouse_sensitivity: f32,
    yaw: f32,
    pitch: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_right_mouse_pressed: false,
            last_mouse_pos: (0.0, 0.0),
            mouse_sensitivity: 0.005,
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
        }
    }

    // Returns true if the event was processed
    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
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
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Right {
                    self.is_right_mouse_pressed = *state == ElementState::Pressed;
                    true
                } else {
                    false
                }
            }
            #[allow(unused_variables)]
            WindowEvent::CursorMoved { position, .. } => self.is_right_mouse_pressed,
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        // Calculate forward vector based on yaw and pitch
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
        let forward =
            glam::Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize();

        // Calculate right vector perpendicular to forward and up
        let right = forward.cross(camera.up).normalize();

        if self.is_forward_pressed {
            camera.eye += forward * self.speed;
            camera.target += forward * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward * self.speed;
            camera.target -= forward * self.speed;
        }
        if self.is_right_pressed {
            camera.eye += right * self.speed;
            camera.target += right * self.speed;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed;
            camera.target -= right * self.speed;
        }
    }

    pub fn process_mouse_movement(&mut self, new_pos: (f32, f32), camera: &mut Camera) {
        if self.is_right_mouse_pressed {
            let delta_x = new_pos.0 - self.last_mouse_pos.0;
            let delta_y = new_pos.1 - self.last_mouse_pos.1;

            // 1. Update Yaw and Pitch based on mouse movement
            self.yaw += delta_x * self.mouse_sensitivity;
            self.pitch -= delta_y * self.mouse_sensitivity;

            // 2. Clamping the pitch
            self.pitch = self.pitch.clamp(-1.55, 1.55);

            // 3. Calculate the new forward vector
            let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
            let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
            let forward =
                glam::Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize();

            // 4. Preserve the distance between eye and target
            let distance = (camera.target - camera.eye).length();
            camera.target = camera.eye + forward * distance;
        }

        self.last_mouse_pos = new_pos;
    }
}
