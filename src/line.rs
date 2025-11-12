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

// Línea con grosor N (1..3 recomendado) usando z-buffer
pub fn line_depth_thick(fb: &mut FrameBuffer, a: &Vec3, b: &Vec3, thickness: i32) {
    let t = thickness.max(1);
    let (x1, y1, z1) = (a.x, a.y, a.z);
    let (x2, y2, z2) = (b.x, b.y, b.z);
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dz = z2 - z1;

    let steps = dx.abs().max(dy.abs()).max(1.0);
    let sx = dx / steps;
    let sy = dy / steps;
    let sz = dz / steps;

    let (mut x, mut y, mut z) = (x1, y1, z1);
    for _ in 0..=steps as i32 {
        let xi = x.round() as i32;
        let yi = y.round() as i32;
        for ox in -t..=t {
            for oy in -t..=t {
                // pequeño disco cuadrado; si quieres más “redondo”, filtra por ox*ox+oy*oy <= t*t
                fb.set_pixel_z(xi + ox, yi + oy, z);
            }
        }
        x += sx;
        y += sy;
        z += sz;
    }
}
