mod camera;
mod framebuffer;
mod geom;
mod line;
mod obj;
mod pixel;
mod setup;
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
use crate::transform::project_vertices_perspective;
use crate::triangle::triangle_filled;

const WIDTH: i32 = 1000;
const HEIGHT: i32 = 700;
// Toggle de luz (Lambert). En false: colores planos.
const APPLY_LIGHT: bool = false;

fn main() -> anyhow::Result<()> {
    // ---------- Ventana ----------
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Software Renderer – Milano (flat + outline)")
        .build();
    rl.set_target_fps(120);

    let mut fb = FrameBuffer::new(WIDTH, HEIGHT, Color::BLACK);

    // ---------- Cargar modelo ----------
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/Nave_espacial.obj".to_string());
    let (vertices, faces) = load_obj(&path)?;
    println!("Vértices: {} | Caras (triángulos): {}", vertices.len(), faces.len());

    // ---------- Bounding box / centro / tamaño ----------
    let (mut min, mut max) = (
        glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        glm::vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    );
    for v in &vertices {
        min.x = min.x.min(v.x); min.y = min.y.min(v.y); min.z = min.z.min(v.z);
        max.x = max.x.max(v.x); max.y = max.y.max(v.y); max.z = max.z.max(v.z);
    }
    let center = (min + max) * 0.5;
    let size   = max - min;
    let r = 0.5 * (size.x * size.x + size.y * size.y + size.z * size.z).sqrt();

    // ---------- Adyacencia arista->caras (para contorno) ----------
    let mut edge_to_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, f) in faces.iter().enumerate() {
        let (i0, i1, i2) = (f.vertex_indices[0], f.vertex_indices[1], f.vertex_indices[2]);
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_faces.entry(key).or_default().push(fi);
        }
    }

    // ---------- Cámara ----------
    let mut cam = Camera::default();
    let aspect = (WIDTH as f32) / (HEIGHT as f32);

    let mut dist = if (cam.fov_y * 0.5).tan() > 1e-6 { r / (cam.fov_y * 0.5).tan() } else { r + 1.0 };
    dist *= 1.2;
    cam.pos = glm::vec3(0.0, 0.0, dist);
    cam.zfar = (dist + r * 2.0).max(1000.0);

    // ---------- Paleta Milano ----------
    let MILANO_BLUE   = Color::new( 25,130,220,255); // alas + bandas laterales
    let MILANO_ORANGE = Color::new(238,120,  0,255); // franja/nariz + acentos
    let MILANO_SILVER = Color::new(200,200,200,255); // cuerpo/underside

    // ---------- Heurísticas de zonas ----------
    // Coordenadas normalizadas en modelo: u ~ X, v ~ Z (hacia la nariz).
    let wing_u_thresh_base   = 0.24;  // más azul entrando al cuerpo
    let wing_u_curve         = 0.08;  // ala “entra” hacia la punta
    let stripe_v_start       = 0.05;  // arranque de la V naranja
    let stripe_u_base        = 0.05;  // ancho mínimo de la franja
    let stripe_u_gain        = 0.30;  // apertura de la V hacia la nariz

    // Banda azul lateral junto a la franja (más azul en cuerpo)
    let blue_side_band_base  = 0.03;
    let blue_side_band_gain  = 0.10;

    // Acento naranja delgado fuera de la banda azul
    let orange_acc_offset    = 0.02;
    let orange_acc_width     = 0.018;
    let orange_acc_gain      = 0.05;
    let orange_acc_v_min     = 0.12;
    let orange_acc_v_max     = 0.48;

    // Nariz hacia +Z o -Z
    let front_dir = if (max.z - center.z) >= (center.z - min.z) { 1.0 } else { -1.0 };

    // Vectores para underside y (si se re-activa) luz
    let light_dir = glm::normalize(&glm::vec3(0.3, 0.6, -1.0));
    let up_view   = glm::vec3(0.0, 1.0, 0.0);

    // Selector de color por triángulo (modelo)
    let choose_milano_color = |va: Vec3, vb: Vec3, vc: Vec3, n_view: glm::Vec3| -> Color {
        // Centroide normalizado
        let u = (((va.x + vb.x + vc.x) / 3.0) - center.x) / size.x; // -0.5..0.5 aprox
        let v = front_dir * ((((va.z + vb.z + vc.z) / 3.0) - center.z) / size.z);

        // V naranja (central)
        let dv = (v - stripe_v_start).max(0.0);
        let stripe_u       = stripe_u_base + stripe_u_gain * dv;

        // Banda azul lateral
        let stripe_u_outer = stripe_u + blue_side_band_base + blue_side_band_gain * dv;

        // Acento naranja delgado
        let acc_inner = stripe_u_outer + orange_acc_offset + orange_acc_gain * dv;
        let acc_outer = acc_inner + orange_acc_width;

        // Ala azul (externa) con curva
        let wing_u_thresh = wing_u_thresh_base - wing_u_curve * v.clamp(0.0, 0.5);

        // Underside (caras que miran hacia abajo): plateado
        if glm::dot(&n_view, &(-up_view)) > 0.35 {
            return MILANO_SILVER;
        }

        // Prioridad: franja/nariz > acento naranja > azul (ala/banda) > cuerpo
        if v > stripe_v_start && u.abs() < stripe_u {
            MILANO_ORANGE
        } else if v > orange_acc_v_min && v < orange_acc_v_max
               && u.abs() >= acc_inner && u.abs() < acc_outer {
            MILANO_ORANGE
        } else if u.abs() > wing_u_thresh
               || (v > stripe_v_start && u.abs() >= stripe_u && u.abs() < stripe_u_outer) {
            MILANO_BLUE
        } else {
            MILANO_SILVER
        }
    };

    // ---------- Loop ----------
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        cam.update_input(&rl, dt);

        // Matrices
        let model = glm::translation(&(-center));
        let view  = cam.view_matrix();
        let proj  = cam.proj_matrix(aspect);
        let mvp   = proj * view * model;
        let mv    = view * model;

        // Vértices proyectados a pantalla (para raster)
        let screen_vertices: Vec<Vec3> =
            project_vertices_perspective(&vertices, &mvp, WIDTH, HEIGHT);

        // Vértices en espacio de vista (para normales y front/back)
        let view_vertices: Vec<glm::Vec3> = vertices.iter().map(|v| {
            let p = mv * glm::vec4(v.x, v.y, v.z, 1.0);
            glm::vec3(p.x, p.y, p.z)
        }).collect();

        // Caras front-facing (para contorno)
        let mut is_front = vec![false; faces.len()];
        for (fi, f) in faces.iter().enumerate() {
            let (i0, i1, i2) = (f.vertex_indices[0], f.vertex_indices[1], f.vertex_indices[2]);
            let va = view_vertices[i0];
            let vb = view_vertices[i1];
            let vc = view_vertices[i2];
            let n = glm::normalize(&glm::cross(&(vb - va), &(vc - va)));
            is_front[fi] = n.z < 0.0; // invierte si tu cámara mira +Z
        }

        fb.clear();

        // ---------- Relleno ----------
        for f in &faces {
            let (i0, i1, i2) = (f.vertex_indices[0], f.vertex_indices[1], f.vertex_indices[2]);

            // pantalla
            let a = screen_vertices[i0];
            let b = screen_vertices[i1];
            let c = screen_vertices[i2];

            // modelo/vista
            let va_m = vertices[i0];
            let vb_m = vertices[i1];
            let vc_m = vertices[i2];
            let va_v = view_vertices[i0];
            let vb_v = view_vertices[i1];
            let vc_v = view_vertices[i2];
            let n_view = glm::normalize(&glm::cross(&(vb_v - va_v), &(vc_v - va_v)));

            let base = choose_milano_color(va_m, vb_m, vc_m, n_view);

            // --- SIN LUZ: k = 1.0 (colores planos) ---
            let k = if APPLY_LIGHT {
                // (opcional) Luz simple si se activa el switch
                let ab = glm::vec3(b.x - a.x, b.y - a.y, b.z - a.z);
                let ac = glm::vec3(c.x - a.x, c.y - a.y, c.z - a.z);
                let n  = glm::normalize(&glm::cross(&ab, &ac));
                let intensity = 0.0f32.max(glm::dot(&n, &(-light_dir)));
                0.35 + 0.65 * intensity
            } else {
                1.0
            };

            let rf = (base.r as f32 * k).clamp(0.0, 255.0) as u8;
            let gf = (base.g as f32 * k).clamp(0.0, 255.0) as u8;
            let bf = (base.b as f32 * k).clamp(0.0, 255.0) as u8;

            fb.set_color(Color::new(rf, gf, bf, 255));
            triangle_filled(&mut fb, &a, &b, &c);
        }

        // ---------- Contorno exterior ----------
        fb.set_color(Color::BLACK);
        let thickness = 2;
        for (&(i0, i1), adj) in &edge_to_faces {
            let draw = match adj.as_slice() {
                [f0]      => is_front[*f0],
                [f0, f1]  => is_front[*f0] ^ is_front[*f1],
                _         => false,
            };
            if draw {
                let a = screen_vertices[i0];
                let b = screen_vertices[i1];
                line_depth_thick(&mut fb, &a, &b, thickness);
            }
        }

        // ---------- Presentación ----------
        let tex = rl
            .load_texture_from_image(&thread, &fb.color_buffer)
            .expect("No pude crear Texture2D desde el framebuffer");
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex, 0, 0, Color::WHITE);
        d.draw_text("WASD/QE mover | Mouse/Flechas mirar | Z/X FOV | M mouse on/off | P PNG",
                    10, 10, 16, Color::RAYWHITE);

        if d.is_key_pressed(KeyboardKey::KEY_P) {
            if let Err(e) = fb.render_to_file("render.png") {
                eprintln!("Error guardando PNG: {e}");
            } else { println!("Guardado: render.png"); }
        }
    }

    Ok(())
}
