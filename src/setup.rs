use crate::geom::Vec3;
use crate::obj::Face;

pub fn setup_vertex_array(vertices: &[Vec3], faces: &[Face]) -> Vec<Vec3> {
    let mut out = Vec::with_capacity(faces.len() * 3);
    for f in faces {
        let a = vertices[f.vertex_indices[0]];
        let b = vertices[f.vertex_indices[1]];
        let c = vertices[f.vertex_indices[2]];
        out.push(a);
        out.push(b);
        out.push(c);
    }
    out
}
