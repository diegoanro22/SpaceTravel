use crate::framebuffer::FrameBuffer;
use crate::geom::Vec3;

#[inline]
pub fn point(fb: &mut FrameBuffer, p: &Vec3) {
    let x = p.x.round() as i32;
    let y = p.y.round() as i32;
    fb.set_pixel(x, y);
}
