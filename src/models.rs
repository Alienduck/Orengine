use crate::{error::Result, vertex::Vertex};
use std::path::Path;

pub fn load_model(file_name: &str) -> Result<(Vec<Vertex>, Vec<u32>)> {
    let path = Path::new("assets").join(file_name);

    // 1. Load the OBJ file
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let (models, materials) = tobj::load_obj(&path, &load_options)?;

    let _materials = materials?;

    // We assume the file contains only one object
    let mesh = &models[0].mesh;

    // 2. Convert positions to our Vertex format
    let mut vertices = Vec::new();

    // Positions are flat: [x, y, z, x, y, z, ...]
    for i in 0..mesh.positions.len() / 3 {
        let tex_coords = if mesh.texcoords.len() > i * 2 {
            [
                mesh.texcoords[i * 2],
                1.0 - mesh.texcoords[i * 2 + 1], // Flip V (Y) for wgpu
            ]
        } else {
            [0.0, 0.0]
        };

        let normal = if mesh.normals.len() > i * 3 {
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
            color: [1.0, 1.0, 1.0],
            tex_coords,
            normal,
        });
    }

    // 3. Get indices directly
    let indices: Vec<u32> = mesh.indices.clone();

    Ok((vertices, indices))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::OrengineError;

    #[test]
    fn test_load_model_not_found() {
        let result = load_model("non_existent_model.obj");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, OrengineError::Tobj(_)));
    }
}
