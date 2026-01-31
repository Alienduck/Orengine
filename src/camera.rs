use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: glam::Vec3,
    pub direction: glam::Vec3,
}

impl Ray {
    /// Checks for intersection with an Axis-Aligned Bounding Box (AABB).
    /// Returns the distance to the intersection, or None if no intersection.
    pub fn intersect_aabb(&self, min: glam::Vec3, max: glam::Vec3) -> Option<f32> {
        let t1 = (min.x - self.origin.x) / self.direction.x;
        let t2 = (max.x - self.origin.x) / self.direction.x;
        let t3 = (min.y - self.origin.y) / self.direction.y;
        let t4 = (max.y - self.origin.y) / self.direction.y;
        let t5 = (min.z - self.origin.z) / self.direction.z;
        let t6 = (max.z - self.origin.z) / self.direction.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some(tmin.max(0.0))
        }
    }

    /// Checks for intersection with a Triangle defined by v0, v1, v2.
    /// Returns the distance to the intersection, or None.
    /// Uses Möller–Trumbore intersection algorithm.
    pub fn intersect_triangle(&self, v0: glam::Vec3, v1: glam::Vec3, v2: glam::Vec3) -> Option<f32> {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = self.direction.cross(edge2);
        let a = edge1.dot(h);

        // Ray is parallel to the triangle
        if a > -f32::EPSILON && a < f32::EPSILON {
            return None;
        }

        let f = 1.0 / a;
        let s = self.origin - v0;
        let u = f * s.dot(h);

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * self.direction.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);
        if t > f32::EPSILON {
            Some(t)
        } else {
            None
        }
    }
}

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

    /// Creates a Ray from screen coordinates (pixels)
    pub fn create_ray(&self, screen_pos: glam::Vec2, screen_size: glam::Vec2) -> Ray {
        // Convert screen position to Normalized Device Coordinates (NDC)
        // NDC range is -1.0 to 1.0
        let ndc_x = (screen_pos.x / screen_size.x) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_pos.y / screen_size.y) * 2.0;

        let ndc = glam::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);

        // Unproject to World Space
        let view_proj = self.build_view_projection_matrix();
        let inv_view_proj = view_proj.inverse();

        let mut world_pos = inv_view_proj * ndc;
        world_pos /= world_pos.w;

        let direction =
            (glam::Vec3::new(world_pos.x, world_pos.y, world_pos.z) - self.eye).normalize();

        Ray {
            origin: self.eye,
            direction,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_pos: [f32; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use glam::Mat4;
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            view_pos: [0.0; 4],
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
        // We use [x, y, z, 1.0] to align with 16 bytes (vec4)
        self.view_pos = [camera.eye.x, camera.eye.y, camera.eye.z, 1.0];
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
