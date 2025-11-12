use crate::framebuffer::FrameBuffer;
use crate::geom::Vec3;
use crate::line::line;

#[inline]
pub fn triangle(fb: &mut FrameBuffer, a: &Vec3, b: &Vec3, c: &Vec3) {
    line(fb, a, b);
    line(fb, b, c);
    line(fb, c, a);
}

pub fn triangle_filled(fb: &mut FrameBuffer, a: &Vec3, b: &Vec3, c: &Vec3) {
    // evita NaN/Inf que rompen el bbox
    if !a.x.is_finite()
        || !a.y.is_finite()
        || !a.z.is_finite()
        || !b.x.is_finite()
        || !b.y.is_finite()
        || !b.z.is_finite()
        || !c.x.is_finite()
        || !c.y.is_finite()
        || !c.z.is_finite()
    {
        return;
    }

    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min((fb.width - 1) as f32) as i32;
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min((fb.height - 1) as f32) as i32;

    let den = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    if den.abs() < 1e-6 {
        return;
    }

    let eps = 1e-6; // tolerancia para evitar agujeros por redondeo

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            // barycentric (orientación-invariante)
            let w0 = ((b.y - c.y) * (px - c.x) + (c.x - b.x) * (py - c.y)) / den;
            let w1 = ((c.y - a.y) * (px - c.x) + (a.x - c.x) * (py - c.y)) / den;
            let w2 = 1.0 - w0 - w1;

            if w0 >= -eps && w1 >= -eps && w2 >= -eps {
                // interpola z en NDC
                let z = w0 * a.z + w1 * b.z + w2 * c.z;
                fb.set_pixel_z(x, y, z);
            }
        }
    }
}
