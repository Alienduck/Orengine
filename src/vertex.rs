// This file is for creating vertex which are uses for shaders

#[repr(C)] // Force Rust to use C memory layout (crucial for GPU compatibility, thx Microslop)
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// Position of the vertex in x, y, z
    pub position: [f32; 3],
    /// Color of the vertex in r, g, b
    pub color: [f32; 3],
    /// UV maping coordonate
    pub tex_coords: [f32; 2],
}

// TODO: useless for now
pub const VERTICES: &[Vertex] = &[
    // 0. Top Left - Red
    Vertex {
        position: [-0.2, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    // 1. Bottom Left - Green
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    // 2. Bottom Right - Blue
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    // 3. Top Right - Yellow (Mix of Red and Green)
    Vertex {
        position: [0.5, 0.5, 0.0],
        color: [1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
];

// TODO: useless for now
// The indices (Order to connect the dots)
// We use u16 because we have few points. For huge models, use u32.
pub const INDICES: &[u16] = &[
    0, 1, 2, // First triangle (TopLeft -> BottomLeft -> BottomRight)
    0, 2, 3, // Second triangle (TopLeft -> BottomRight -> TopRight)
];
