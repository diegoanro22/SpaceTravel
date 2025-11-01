use glm::{Mat4, Vec3, Vec4};
use nalgebra_glm as glm;

pub fn project_vertices_perspective(
    verts: &[Vec3],
    mvp: &Mat4,
    width: i32,
    height: i32,
) -> Vec<Vec3> {
    let (w, h) = (width as f32, height as f32);
    verts
        .iter()
        .map(|v| {
            let p = glm::vec4(v.x, v.y, v.z, 1.0);
            let clip: Vec4 = mvp * p;
            let ndc = if clip.w.abs() > 1e-6 {
                glm::vec3(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w)
            } else {
                glm::vec3(0.0, 0.0, 0.0)
            };
            let sx = (ndc.x * 0.5 + 0.5) * w;
            let sy = (1.0 - (ndc.y * 0.5 + 0.5)) * h;
            glm::vec3(sx, sy, ndc.z)
        })
        .collect()
}
