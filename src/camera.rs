use glam::{Mat4, Vec3};

// We need this struct to match the layout in the shader EXACTLY.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4], // 4x4 Matrix
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    // Helper to update the matrix with math logic
    pub fn update_view_proj(&mut self, rotation_angle: f32, aspect_ratio: f32) {
        // 1. Projection: How the camera "sees" (Perspective here for simplicity first)
        let projection = Mat4::perspective_rh(
            45.0_f32.to_radians(), // Field of View
            aspect_ratio,          // Width / Height
            0.1,                   // Near plane
            100.0,                 // Far plane
        );

        // 2. View: Where the camera is
        let view = Mat4::look_at_rh(
            Vec3::new(2.0, 2.0, 2.0), // Camera is at (2, 2, 2)
            Vec3::ZERO,               // Looking at (0, 0, 0)
            Vec3::Y,                  // "Up" is Y axis
        );

        // 3. Model: The object's rotation (Spinning on Z axis)
        let model = Mat4::from_rotation_z(rotation_angle);

        // Combine: Proj * View * Model (Order matters!)
        let combined = projection * view * model;

        self.view_proj = combined.to_cols_array_2d();
    }
}
