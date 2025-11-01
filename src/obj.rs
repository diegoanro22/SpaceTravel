
use anyhow::{anyhow, Result};
use nalgebra_glm as glm;
use crate::geom::Vec3;

#[derive(Debug, Clone)]
pub struct Face {
    pub vertex_indices: [usize; 3],
}

pub fn load_obj(path: &str) -> Result<(Vec<Vec3>, Vec<Face>)> {
    let text = std::fs::read_to_string(path)?;
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();

    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        if line.starts_with("v ") {
            let parts: Vec<_> = line[2..].split_whitespace().collect();
            if parts.len() < 3 { return Err(anyhow!("Línea v inválida @{}: {}", lineno+1, line)); }
            let x: f32 = parts[0].parse()?;
            let y: f32 = parts[1].parse()?;
            let z: f32 = parts[2].parse()?;
            vertices.push(glm::vec3(x, y, z));
        } else if line.starts_with("f ") {
            let toks: Vec<_> = line[2..].split_whitespace().collect();
            if toks.len() < 3 { return Err(anyhow!("Cara con <3 vértices @{}", lineno+1)); }

            let idx_from_token = |tok: &str| -> Result<usize> {
                let first = tok.split('/').next().ok_or_else(|| anyhow!("Token f inválido"))?;
                let i: i32 = first.parse()?;
                let idx = if i > 0 { (i - 1) as usize } else {
                    let len = vertices.len() as i32;
                    (len + i) as usize
                };
                Ok(idx)
            };

            let indices: Result<Vec<_>> = toks.iter().map(|t| idx_from_token(t)).collect();
            let indices = indices?;

            if indices.len() == 3 {
                faces.push(Face { vertex_indices: [indices[0], indices[1], indices[2]] });
            } else if indices.len() == 4 {
                faces.push(Face { vertex_indices: [indices[0], indices[1], indices[2]] });
                faces.push(Face { vertex_indices: [indices[0], indices[2], indices[3]] });
            } else {
                for i in 1..indices.len()-1 {
                    faces.push(Face { vertex_indices: [indices[0], indices[i], indices[i+1]] });
                }
            }
        }
    }

    if vertices.is_empty() { return Err(anyhow!("Sin vértices en {}", path)); }
    if faces.is_empty() { return Err(anyhow!("Sin caras en {}", path)); }

    Ok((vertices, faces))
}
