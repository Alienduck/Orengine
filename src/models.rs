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

    let materials = materials.expect("Failed to load OBJ");

    // We assume the file contains only one object (the pizza)
    // If there are multiple, we take the first one
    let mesh = &models[0].mesh;

    // 2. Convert positions to our Vertex format
    let mut vertices = Vec::new();

    // Positions are flat: [x, y, z, x, y, z, ...]
    for i in 0..mesh.positions.len() / 3 {
        vertices.push(Vertex {
            position: [
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            ],
            // Random-ish color or White since OBJ has no vertex color usually
            color: [1.0, 1.0, 1.0], // White Pizza
        });
    }

    for (i, m) in materials.iter().enumerate() {
        dbg!("Material[{}].name = \'{}\'", i, m.name.as_ref() as &str);
        // TODO: handle materials here
    }

    // 3. Get indices directly (convert to u32 just in case)
    let indices: Vec<u32> = mesh.indices.clone();

    (vertices, indices)
}
