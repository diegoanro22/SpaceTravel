
use nalgebra_glm as glm;

pub type Vec2 = glm::Vec2;
pub type Vec3 = glm::Vec3;
pub type Vec4 = glm::Vec4;

#[inline]
pub fn print_vertex(v: &Vec3) {
    println!("vertex: ({:.4}, {:.4}, {:.4})", v.x, v.y, v.z);
}
