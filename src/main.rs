mod camera;
mod framebuffer;
mod geom;
mod line;
mod obj;
mod pixel;
mod setup;
mod shaders;
mod transform;
mod triangle;

use nalgebra_glm as glm;
use raylib::prelude::*;
use std::collections::HashMap;

use crate::camera::Camera;
use crate::framebuffer::FrameBuffer;
use crate::geom::Vec3;
use crate::line::line_depth_thick;
use crate::obj::load_obj;
use crate::shaders::{BodyShader, shade_body};
use crate::transform::project_vertices_perspective;
use crate::triangle::triangle_filled;

const WIDTH: i32 = 1000;
const HEIGHT: i32 = 700;

//
// ----- JÚPITER: ANILLOS -----
//

fn draw_jupiter_rings(fb: &mut FrameBuffer, mvp: &glm::Mat4, inner_radius: f32, outer_radius: f32) {
    let segments = 128;
    let w = WIDTH as f32;
    let h = HEIGHT as f32;

    let project = |v: glm::Vec3| -> Vec3 {
        let p = glm::vec4(v.x, v.y, v.z, 1.0);
        let clip = *mvp * p;
        if clip.w.abs() < 1e-6 {
            return glm::vec3(-9999.0, -9999.0, 1.0);
        }
        let ndc = glm::vec3(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w);
        let sx = (ndc.x * 0.5 + 0.5) * w;
        let sy = (1.0 - (ndc.y * 0.5 + 0.5)) * h;
        glm::vec3(sx, sy, ndc.z)
    };

    let mut inner_pts = Vec::with_capacity(segments + 1);
    let mut outer_pts = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let a = t * std::f32::consts::TAU;

        let v_inner = glm::vec3(inner_radius * a.cos(), 0.0, inner_radius * a.sin());
        let v_outer = glm::vec3(outer_radius * a.cos(), 0.0, outer_radius * a.sin());

        inner_pts.push(project(v_inner));
        outer_pts.push(project(v_outer));
    }

    for i in 0..segments {
        let p0 = inner_pts[i];
        let p1 = outer_pts[i];
        let p2 = inner_pts[i + 1];
        let p3 = outer_pts[i + 1];

        let t = i as f32 / segments as f32;
        let stripe = (t * 12.0).sin() * 0.5 + 0.5;

        let base = glm::vec3(0.78, 0.80, 0.90);
        let accent = glm::vec3(0.35, 0.85, 1.20);
        let col = base * (1.0 - stripe * 0.5) + accent * (stripe * 0.9);

        let r = (col.x.clamp(0.0, 1.4) * 255.0) as u8;
        let g = (col.y.clamp(0.0, 1.4) * 255.0) as u8;
        let b = (col.z.clamp(0.0, 1.4) * 255.0) as u8;
        fb.set_color(Color::new(r, g, b, 255));

        triangle_filled(fb, &p0, &p1, &p2);
        triangle_filled(fb, &p2, &p1, &p3);
    }

    fb.set_color(Color::new(230, 240, 255, 255));

    for i in 0..segments {
        let a = inner_pts[i];
        let b = inner_pts[i + 1];
        line_depth_thick(fb, &a, &b, 1);
    }
    for i in 0..segments {
        let a = outer_pts[i];
        let b = outer_pts[i + 1];
        line_depth_thick(fb, &a, &b, 1);
    }
}

//
// ----- MAIN -----
//

fn main() -> anyhow::Result<()> {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Software Renderer – Mini Sistema Solar")
        .build();
    rl.set_target_fps(120);

    let mut fb = FrameBuffer::new(WIDTH, HEIGHT, Color::BLACK);

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/sphere.obj".to_string());
    let (vertices, faces) = load_obj(&path)?;
    println!(
        "Vértices: {} | Caras (triángulos): {}",
        vertices.len(),
        faces.len()
    );

    let (mut min, mut max) = (
        glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        glm::vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    );
    for v in &vertices {
        min.x = min.x.min(v.x);
        min.y = min.y.min(v.y);
        min.z = min.z.min(v.z);
        max.x = max.x.max(v.x);
        max.y = max.y.max(v.y);
        max.z = max.z.max(v.z);
    }
    let center = (min + max) * 0.5;
    let size = max - min;
    let r_bb = 0.5 * (size.x * size.x + size.y * size.y + size.z * size.z).sqrt();
    let sphere_radius = 0.5 * size.x.max(size.y).max(size.z);

    let mut edge_to_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, f) in faces.iter().enumerate() {
        let (i0, i1, i2) = (
            f.vertex_indices[0],
            f.vertex_indices[1],
            f.vertex_indices[2],
        );
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_faces.entry(key).or_default().push(fi);
        }
    }

    let mut cam = Camera::default();
    let aspect = (WIDTH as f32) / (HEIGHT as f32);

    let mut dist = if (cam.fov_y * 0.5).tan() > 1e-6 {
        r_bb / (cam.fov_y * 0.5).tan()
    } else {
        r_bb + 1.0
    };
    dist *= 1.8;
    cam.pos = glm::vec3(0.0, 0.0, dist);
    cam.zfar = (dist + r_bb * 2.0).max(1000.0);

    let mut current_shader = BodyShader::Star;
    let mut time_acc: f32 = 0.0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time_acc += dt;
        cam.update_input(&rl, dt);

        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            current_shader = BodyShader::Star;
            println!("Shader: Sol");
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            current_shader = BodyShader::Mercury;
            println!("Shader: Mercurio");
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            current_shader = BodyShader::Venus;
            println!("Shader: Venus");
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            current_shader = BodyShader::Rocky;
            println!("Shader: Tierra + Luna");
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FIVE) {
            current_shader = BodyShader::Mars;
            println!("Shader: Marte");
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SIX) {
            current_shader = BodyShader::GasGiant;
            println!("Shader: Júpiter + Anillos");
        }

        let view = cam.view_matrix();
        let proj = cam.proj_matrix(aspect);

        let rot = glm::rotation(time_acc * 0.25, &glm::vec3(0.0, 1.0, 0.0));

        //
        // ----- PLANETA PRINCIPAL -----
        //

        let model_planet = rot * glm::translation(&(-center));
        let mvp_planet = proj * view * model_planet;
        let mv_planet = view * model_planet;

        let screen_planet: Vec<Vec3> =
            project_vertices_perspective(&vertices, &mvp_planet, WIDTH, HEIGHT);

        let view_planet: Vec<glm::Vec3> = vertices
            .iter()
            .map(|v| {
                let p = mv_planet * glm::vec4(v.x, v.y, v.z, 1.0);
                glm::vec3(p.x, p.y, p.z)
            })
            .collect();

        let mut is_front_planet = vec![false; faces.len()];
        for (fi, f) in faces.iter().enumerate() {
            let (i0, i1, i2) = (
                f.vertex_indices[0],
                f.vertex_indices[1],
                f.vertex_indices[2],
            );
            let va = view_planet[i0];
            let vb = view_planet[i1];
            let vc = view_planet[i2];
            let n = glm::normalize(&glm::cross(&(vb - va), &(vc - va)));
            is_front_planet[fi] = n.z < 0.0;
        }

        //
        // ----- LUNA (SOLO TIERRA) -----
        //

        let moon_scale = 0.35;
        let moon_orbit_radius = sphere_radius * 2.6;
        let moon_angle = time_acc * 0.6;

        let moon_offset = glm::vec3(
            moon_orbit_radius * moon_angle.cos(),
            sphere_radius * 0.3 * (moon_angle * 0.8).sin(),
            moon_orbit_radius * moon_angle.sin(),
        );

        let model_moon = glm::translation(&moon_offset)
            * rot
            * glm::scaling(&glm::vec3(moon_scale, moon_scale, moon_scale))
            * glm::translation(&(-center));

        let mvp_moon = proj * view * model_moon;
        let mv_moon = view * model_moon;

        let screen_moon: Vec<Vec3> =
            project_vertices_perspective(&vertices, &mvp_moon, WIDTH, HEIGHT);

        let view_moon: Vec<glm::Vec3> = vertices
            .iter()
            .map(|v| {
                let p = mv_moon * glm::vec4(v.x, v.y, v.z, 1.0);
                glm::vec3(p.x, p.y, p.z)
            })
            .collect();

        let mut is_front_moon = vec![false; faces.len()];
        for (fi, f) in faces.iter().enumerate() {
            let (i0, i1, i2) = (
                f.vertex_indices[0],
                f.vertex_indices[1],
                f.vertex_indices[2],
            );
            let va = view_moon[i0];
            let vb = view_moon[i1];
            let vc = view_moon[i2];
            let n = glm::normalize(&glm::cross(&(vb - va), &(vc - va)));
            is_front_moon[fi] = n.z < 0.0;
        }

        fb.clear();

        //
        // ----- FILL PLANETA -----
        //

        for f in &faces {
            let (i0, i1, i2) = (
                f.vertex_indices[0],
                f.vertex_indices[1],
                f.vertex_indices[2],
            );

            let a = screen_planet[i0];
            let b = screen_planet[i1];
            let c = screen_planet[i2];

            let va_m = vertices[i0];
            let vb_m = vertices[i1];
            let vc_m = vertices[i2];
            let va_v = view_planet[i0];
            let vb_v = view_planet[i1];
            let vc_v = view_planet[i2];

            let n_view = glm::normalize(&glm::cross(&(vb_v - va_v), &(vc_v - va_v)));
            let centroid = (va_m + vb_m + vc_m) / 3.0;

            let color = shade_body(
                current_shader,
                centroid,
                n_view,
                center,
                sphere_radius,
                time_acc,
            );

            fb.set_color(color);
            triangle_filled(&mut fb, &a, &b, &c);
        }

        //
        // ----- FILL LUNA (TIERRA) -----
        //

        if let BodyShader::Rocky = current_shader {
            for f in &faces {
                let (i0, i1, i2) = (
                    f.vertex_indices[0],
                    f.vertex_indices[1],
                    f.vertex_indices[2],
                );

                let a = screen_moon[i0];
                let b = screen_moon[i1];
                let c = screen_moon[i2];

                let va_m = vertices[i0];
                let vb_m = vertices[i1];
                let vc_m = vertices[i2];
                let va_v = view_moon[i0];
                let vb_v = view_moon[i1];
                let vc_v = view_moon[i2];

                let n_view = glm::normalize(&glm::cross(&(vb_v - va_v), &(vc_v - va_v)));
                let centroid = (va_m + vb_m + vc_m) / 3.0;

                let color = shade_body(
                    BodyShader::Moon,
                    centroid,
                    n_view,
                    center,
                    sphere_radius,
                    time_acc,
                );

                fb.set_color(color);
                triangle_filled(&mut fb, &a, &b, &c);
            }
        }

        //
        // ----- JÚPITER: ANILLOS -----
        //

        if let BodyShader::GasGiant = current_shader {
            let inner = sphere_radius * 1.4;
            let outer = sphere_radius * 2.4;
            draw_jupiter_rings(&mut fb, &mvp_planet, inner, outer);
        }

        //
        // ----- CONTORNO PLANETA -----
        //

        fb.set_color(Color::BLACK);
        let thickness = 2;
        for (&(i0, i1), adj) in &edge_to_faces {
            let draw = match adj.as_slice() {
                [f0] => is_front_planet[*f0],
                [f0, f1] => is_front_planet[*f0] ^ is_front_planet[*f1],
                _ => false,
            };
            if draw {
                let a = screen_planet[i0];
                let b = screen_planet[i1];
                line_depth_thick(&mut fb, &a, &b, thickness);
            }
        }

        //
        // ----- CONTORNO LUNA (TIERRA) -----
        //

        if let BodyShader::Rocky = current_shader {
            fb.set_color(Color::BLACK);
            for (&(i0, i1), adj) in &edge_to_faces {
                let draw = match adj.as_slice() {
                    [f0] => is_front_moon[*f0],
                    [f0, f1] => is_front_moon[*f0] ^ is_front_moon[*f1],
                    _ => false,
                };
                if draw {
                    let a = screen_moon[i0];
                    let b = screen_moon[i1];
                    line_depth_thick(&mut fb, &a, &b, thickness);
                }
            }
        }

        //
        // ----- PRESENTACIÓN -----
        //

        let tex = rl
            .load_texture_from_image(&thread, &fb.color_buffer)
            .expect("No pude crear Texture2D desde el framebuffer");
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex, 0, 0, Color::WHITE);
        d.draw_text(
            "1=Sol  2=Mercurio  3=Venus  4=Tierra+Luna  5=Marte  6=Jupiter+Anillos | WASD/QE mover | Mouse/Flechas mirar | Z/X FOV | P PNG",
            10,
            10,
            16,
            Color::RAYWHITE,
        );

        if d.is_key_pressed(KeyboardKey::KEY_P) {
            if let Err(e) = fb.render_to_file("render.png") {
                eprintln!("Error guardando PNG: {e}");
            } else {
                println!("Guardado: render.png");
            }
        }
    }

    Ok(())
}
