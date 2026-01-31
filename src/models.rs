use crate::{error::Result, vertex::Vertex};
use std::{fmt::Debug, path::Path};

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: String,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_id: usize,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub aabb: Aabb,
}

impl Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("meshes", &self.meshes)
            .field("materials", &self.materials)
            .field("aabb", &self.aabb)
            .finish()
    }
}

pub fn load_model(file_name: &str) -> Result<Model> {
    let path = Path::new("assets").join(file_name);

    // 1. Load the OBJ file
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let (models, materials) = tobj::load_obj(&path, &load_options)?;

    let materials = materials?;

    // Convert materials
    let mut out_materials = Vec::new();
    for mat in materials {
        out_materials.push(Material {
            name: mat.name,
            diffuse_texture: mat.diffuse_texture.unwrap_or_default(),
        });
    }

    // Convert meshes
    let mut min_pos = [f32::INFINITY; 3];
    let mut max_pos = [f32::NEG_INFINITY; 3];

    let mut out_meshes = Vec::new();
    for m in models {
        let mesh = m.mesh;
        let mut vertices = Vec::new();

        // Positions are flat: [x, y, z, x, y, z, ...]
        for i in 0..mesh.positions.len() / 3 {
            let x = mesh.positions[i * 3];
            let y = mesh.positions[i * 3 + 1];
            let z = mesh.positions[i * 3 + 2];

            // Update AABB bounds
            if x < min_pos[0] {
                min_pos[0] = x;
            }
            if y < min_pos[1] {
                min_pos[1] = y;
            }
            if z < min_pos[2] {
                min_pos[2] = z;
            }
            if x > max_pos[0] {
                max_pos[0] = x;
            }
            if y > max_pos[1] {
                max_pos[1] = y;
            }
            if z > max_pos[2] {
                max_pos[2] = z;
            }

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
                position: [x, y, z],
                color: [1.0, 1.0, 1.0],
                tex_coords,
                normal,
            });
        }

        out_meshes.push(Mesh {
            name: m.name,
            vertices,
            indices: mesh.indices,
            material_id: mesh.material_id.unwrap_or(0),
        });
    }

    Ok(Model {
        meshes: out_meshes,
        materials: out_materials,
        aabb: Aabb {
            min: min_pos,
            max: max_pos,
        },
    })
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
