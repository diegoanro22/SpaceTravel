use crate::framebuffer::FrameBuffer;
use crate::geom::Vec3;
use crate::pixel::point;

#[inline]
pub fn line(fb: &mut FrameBuffer, start: &Vec3, end: &Vec3) {
    let (mut x1, mut y1) = (start.x.round() as i32, start.y.round() as i32);
    let (x2, y2) = (end.x.round() as i32, end.y.round() as i32);

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        point(fb, &Vec3::new(x1 as f32, y1 as f32, 0.0));
        if x1 == x2 && y1 == y2 {
            break;
        }
        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            x1 += sx;
        }
        if e2 < dx {
            err += dx;
            y1 += sy;
        }
    }
}
