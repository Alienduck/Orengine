use crate::vertex::Vertex;
use std::path::Path;
use tobj;

pub fn load_model(file_name: &str) -> (Vec<Vertex>, Vec<u32>) {
    let path = Path::new("assets").join(file_name);

    // 1. Load the OBJ file
    // triangulate: true -> ensure we only have triangles (no squares/ngons)
    // single_index: true -> unifies indices for pos/norm/texcoord (easier for wgpu)
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let (models, materials) =
        tobj::load_obj(&path, &load_options).expect("Failed to load 3D model");

    let _materials = materials.expect("Failed to load OBJ");

    // We assume the file contains only one object (the pizza)
    // If there are multiple, we take the first one
    let mesh = &models[0].mesh;

    // 2. Convert positions to our Vertex format
    let mut vertices = Vec::new();

    // Positions are flat: [x, y, z, x, y, z, ...]
    for i in 0..mesh.positions.len() / 3 {
        // Handle missing texture coordinates by defaulting to (0.0, 0.0)
        let tex_coords = if mesh.texcoords.len() >= (i + 1) * 2 {
            [
                mesh.texcoords[i * 2],
                1.0 - mesh.texcoords[i * 2 + 1], // Flip V (Y) for wgpu
            ]
        } else {
            [0.0, 0.0]
        };

        // Handle missing normals by using a default
        let normal = if mesh.normals.len() >= (i + 1) * 3 {
            [
                mesh.normals[i * 3],
                mesh.normals[i * 3 + 1],
                mesh.normals[i * 3 + 2],
            ]
        } else {
            [0.0, 1.0, 0.0] // Default upward normal
        };

        vertices.push(Vertex {
            position: [
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            ],
            // White color since OBJ has no vertex color usually
            color: [1.0, 1.0, 1.0],
            tex_coords,
            normal,
        });
    }

    // 3. Get indices directly (convert to u32 just in case)
    let indices: Vec<u32> = mesh.indices.clone();

    (vertices, indices)
}
