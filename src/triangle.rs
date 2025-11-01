use crate::framebuffer::FrameBuffer;
use crate::geom::Vec3;
use crate::line::line;

#[inline]
pub fn triangle(fb: &mut FrameBuffer, a: &Vec3, b: &Vec3, c: &Vec3) {
    line(fb, a, b);
    line(fb, b, c);
    line(fb, c, a);
}
