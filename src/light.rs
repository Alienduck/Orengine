use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16-byte (4 float) spacing, we need padding
    pub _padding: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}
