use winit::event::ElementState;
use winit::keyboard::KeyCode;

pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

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

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,   // For 'A' (Up)
    is_down_pressed: bool, // For 'E' (Down)
    yaw: f32,
    pitch: f32,
    mouse_sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
            mouse_sensitivity: 0.003,
        }
    }

    pub fn process_keyboard(&mut self, keycode: KeyCode, state: ElementState) -> bool {
        let is_pressed = state == ElementState::Pressed;
        match keycode {
            // ZQSD (AZERTY) physically corresponds to WASD (QWERTY)
            // So KeyW = Z, KeyA = Q, KeyS = S, KeyD = D
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                // Key 'Q' on AZERTY (left)
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
            KeyCode::KeyQ => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::KeyE => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.yaw += mouse_dx as f32 * self.mouse_sensitivity;
        self.pitch -= mouse_dy as f32 * self.mouse_sensitivity;
        self.pitch = self.pitch.clamp(-1.54, 1.54);
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        // 1. Recalculate orientation
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();

        let forward =
            glam::Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize();

        // IMPORTANT: We force the target to be in front of the eye according to the new angle
        // This is what "takes control" of the camera
        camera.target = camera.eye + forward;

        // 2. Movements
        let forward_norm = forward.normalize();
        let right_norm = forward_norm.cross(camera.up).normalize();

        if self.is_forward_pressed {
            camera.eye += forward_norm * self.speed;
            camera.target += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
            camera.target -= forward_norm * self.speed;
        }
        if self.is_right_pressed {
            camera.eye += right_norm * self.speed;
            camera.target += right_norm * self.speed;
        }
        if self.is_left_pressed {
            camera.eye -= right_norm * self.speed;
            camera.target -= right_norm * self.speed;
        }

        if self.is_up_pressed {
            camera.eye -= glam::Vec3::Y * self.speed;
            camera.target -= glam::Vec3::Y * self.speed;
        }
        if self.is_down_pressed {
            camera.eye += glam::Vec3::Y * self.speed;
            camera.target += glam::Vec3::Y * self.speed;
        }
    }
}
