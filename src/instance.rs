use glam::{Mat4, Quat, Vec3};

// 1. The "Logic" version (CPU)
// This is what you'll manipulate to place your objects
pub struct Instance {
    pub position: Vec3,
    pub rotation: Quat,
}

impl Instance {
    // Converts logic to raw data for the GPU
    pub fn to_raw(&self) -> InstanceRaw {
        // Creates a transformation matrix: Translation * Rotation
        let model = Mat4::from_rotation_translation(self.rotation, self.position);
        InstanceRaw {
            model: model.to_cols_array_2d(),
        }
    }
}

// 2. The "Raw" version (GPU)
// The GPU wants a 4x4 matrix to know where to draw
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    // This function explains to WGPU how to read this structure in memory
    // It's like VertexBufferLayout but for instances
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // IMPORTANT: We advance one step per INSTANCE, not per VERTEX
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4x4 takes 4 slots of vec4 (4 * 16 bytes)
                // WGPU doesn't allow defining "mat4" directly, must split it

                // Location 5: Row 1
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5, // We start at 5 to leave room for other attributes
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Location 6: Row 2
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Location 7: Row 3
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Location 8: Row 4
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
