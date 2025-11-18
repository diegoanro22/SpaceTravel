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

const STAR_COUNT: usize = 400;

struct PlanetDef {
    kind: BodyShader,
    orbit_radius: f32,
    orbit_speed: f32,
    self_speed: f32,
    scale: f32,
    has_moon: bool,
}

struct Instance {
    kind: BodyShader,
    model: glm::Mat4,
    center_world: glm::Vec3,
    radius_collision: f32,
}

struct WarpState {
    active: bool,
    t: f32,
    start_pos: glm::Vec3,
    end_pos: glm::Vec3,
}

//
// ----- ÓRBITAS PLANETARIAS (líneas finas en el plano eclíptico) -----
//

fn draw_orbit(fb: &mut FrameBuffer, view: &glm::Mat4, proj: &glm::Mat4, radius: f32, color: Color) {
    let segments = 128;
    let w = WIDTH as f32;
    let h = HEIGHT as f32;
    let mvp = proj * view;

    // z fijo MUY LEJOS para que siempre quede detrás de todo en el z-buffer
    let orbit_depth = 10_000.0;

    // Proyecta un punto de la órbita. Si está detrás de la cámara o muy fuera,
    // devolvemos None y NO se dibuja ese tramo.
    let project = |v: glm::Vec3| -> Option<Vec3> {
        let p = glm::vec4(v.x, v.y, v.z, 1.0);
        let clip = mvp * p;

        // Detrás de la cámara o muy cerca del plano cercano
        if clip.w <= 1e-6 {
            return None;
        }

        let ndc_x = clip.x / clip.w;
        let ndc_y = clip.y / clip.w;

        // Si se va MUY fuera de la pantalla, también lo descartamos
        if ndc_x < -2.0 || ndc_x > 2.0 || ndc_y < -2.0 || ndc_y > 2.0 {
            return None;
        }

        let sx = (ndc_x * 0.5 + 0.5) * w;
        let sy = (1.0 - (ndc_y * 0.5 + 0.5)) * h;

        Some(glm::vec3(sx, sy, orbit_depth))
    };

    fb.set_color(color);

    let mut first_valid: Option<Vec3> = None;
    let mut prev: Option<Vec3> = None;

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let a = t * std::f32::consts::TAU;

        // Órbita en el plano y=0 alrededor del origen (sol)
        let v = glm::vec3(radius * a.cos(), 0.0, radius * a.sin());

        let cur = project(v);

        if let Some(p) = cur {
            if first_valid.is_none() {
                first_valid = Some(p);
            }
            if let Some(prev_p) = prev {
                // Solo dibujamos si ambos puntos son válidos
                line_depth_thick(fb, &prev_p, &p, 1);
            }
            prev = Some(p);
        } else {
            // Cortamos la tira: el siguiente segmento empezará desde aquí
            prev = None;
        }
    }

    // Opcional: cerrar el círculo si el último y el primero son válidos
    if let (Some(p0), Some(p_last)) = (first_valid, prev) {
        line_depth_thick(fb, &p_last, &p0, 1);
    }
}

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
// ----- NAVE MILANO: PALETA Y COLOR POR CARA -----
//

fn choose_milano_color(
    va: Vec3,
    vb: Vec3,
    vc: Vec3,
    n_view: glm::Vec3,
    center: glm::Vec3,
    size: glm::Vec3,
) -> Color {
    let milano_blue = Color::new(25, 130, 220, 255);
    let milano_orange = Color::new(238, 120, 0, 255);
    let milano_silver = Color::new(200, 200, 200, 255);

    let up_view = glm::vec3(0.0, 1.0, 0.0);

    let u = (((va.x + vb.x + vc.x) / 3.0) - center.x) / size.x;
    let v = (((va.z + vb.z + vc.z) / 3.0) - center.z) / size.z;

    let wing_u_thresh_base = 0.24;
    let wing_u_curve = 0.08;
    let stripe_v_start = 0.05;
    let stripe_u_base = 0.05;
    let stripe_u_gain = 0.30;
    let blue_side_band_base = 0.03;
    let blue_side_band_gain = 0.10;
    let orange_acc_offset = 0.02;
    let orange_acc_width = 0.018;
    let orange_acc_gain = 0.05;
    let orange_acc_v_min = 0.12;
    let orange_acc_v_max = 0.48;

    if glm::dot(&n_view, &(-up_view)) > 0.35 {
        return milano_silver;
    }

    let dv = (v - stripe_v_start).max(0.0);
    let stripe_u = stripe_u_base + stripe_u_gain * dv;
    let stripe_u_outer = stripe_u + blue_side_band_base + blue_side_band_gain * dv;

    let acc_inner = stripe_u_outer + orange_acc_offset + orange_acc_gain * dv;
    let acc_outer = acc_inner + orange_acc_width;

    let wing_u_thresh = wing_u_thresh_base - wing_u_curve * v.clamp(0.0, 0.5);

    if v > stripe_v_start && u.abs() < stripe_u {
        milano_orange
    } else if v > orange_acc_v_min
        && v < orange_acc_v_max
        && u.abs() >= acc_inner
        && u.abs() < acc_outer
    {
        milano_orange
    } else if u.abs() > wing_u_thresh
        || (v > stripe_v_start && u.abs() >= stripe_u && u.abs() < stripe_u_outer)
    {
        milano_blue
    } else {
        milano_silver
    }
}

//
// ----- MAIN FINAL -----
//

fn main() -> anyhow::Result<()> {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("SpaceTravel – Comic Solar System")
        .build();
    rl.set_target_fps(120);

    let mut fb = FrameBuffer::new(WIDTH, HEIGHT, Color::BLACK);

    // ----- Esfera base (sol/planetas/lunas) -----
    let sphere_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/sphere.obj".to_string());
    let (sphere_vertices, sphere_faces) = load_obj(&sphere_path)?;
    println!(
        "Sphere mesh -> Vértices: {} | Caras: {}",
        sphere_vertices.len(),
        sphere_faces.len()
    );

    let (mut s_min, mut s_max) = (
        glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        glm::vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    );
    for v in &sphere_vertices {
        s_min.x = s_min.x.min(v.x);
        s_min.y = s_min.y.min(v.y);
        s_min.z = s_min.z.min(v.z);
        s_max.x = s_max.x.max(v.x);
        s_max.y = s_max.y.max(v.y);
        s_max.z = s_max.z.max(v.z);
    }
    let sphere_center = (s_min + s_max) * 0.5;
    let s_size = s_max - s_min;
    let sphere_radius =
        0.5 * (s_size.x * s_size.x + s_size.y * s_size.y + s_size.z * s_size.z).sqrt();

    let mut sphere_edge_to_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, f) in sphere_faces.iter().enumerate() {
        let (i0, i1, i2) = (
            f.vertex_indices[0],
            f.vertex_indices[1],
            f.vertex_indices[2],
        );
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            sphere_edge_to_faces.entry(key).or_default().push(fi);
        }
    }

    // ----- Nave Milano -----
    let (ship_vertices, ship_faces) = load_obj("assets/Nave_espacial.obj")?;
    println!(
        "Ship mesh   -> Vértices: {} | Caras: {}",
        ship_vertices.len(),
        ship_faces.len()
    );

    let (mut sh_min, mut sh_max) = (
        glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        glm::vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    );
    for v in &ship_vertices {
        sh_min.x = sh_min.x.min(v.x);
        sh_min.y = sh_min.y.min(v.y);
        sh_min.z = sh_min.z.min(v.z);
        sh_max.x = sh_max.x.max(v.x);
        sh_max.y = sh_max.y.max(v.y);
        sh_max.z = sh_max.z.max(v.z);
    }
    let ship_center = (sh_min + sh_max) * 0.5;
    let ship_size = sh_max - sh_min;

    let mut ship_edge_to_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, f) in ship_faces.iter().enumerate() {
        let (i0, i1, i2) = (
            f.vertex_indices[0],
            f.vertex_indices[1],
            f.vertex_indices[2],
        );
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            ship_edge_to_faces.entry(key).or_default().push(fi);
        }
    }

    // ----- Cámara -----
    let mut cam = Camera::default();
    let aspect = (WIDTH as f32) / (HEIGHT as f32);

    let mut dist = if (cam.fov_y * 0.5).tan() > 1e-6 {
        sphere_radius / (cam.fov_y * 0.5).tan()
    } else {
        sphere_radius + 1.0
    };
    dist *= 6.0;
    cam.pos = glm::vec3(0.0, 8.0, dist);
    cam.zfar = 5000.0;

    // ----- Sistema planetario -----
    let sun_scale = 3.0;

    let orbit_base = sphere_radius * 6.0;
    let planets = [
        PlanetDef {
            kind: BodyShader::Mercury,
            orbit_radius: orbit_base * 1.0,
            orbit_speed: 0.6,
            self_speed: 2.0,
            scale: 0.45,
            has_moon: false,
        },
        PlanetDef {
            kind: BodyShader::Venus,
            orbit_radius: orbit_base * 1.6,
            orbit_speed: 0.45,
            self_speed: 1.6,
            scale: 0.8,
            has_moon: false,
        },
        PlanetDef {
            kind: BodyShader::Rocky,
            orbit_radius: orbit_base * 2.3,
            orbit_speed: 0.35,
            self_speed: 1.8,
            scale: 0.9,
            has_moon: true,
        },
        PlanetDef {
            kind: BodyShader::Mars,
            orbit_radius: orbit_base * 3.1,
            orbit_speed: 0.25,
            self_speed: 1.5,
            scale: 0.75,
            has_moon: false,
        },
        PlanetDef {
            kind: BodyShader::GasGiant,
            orbit_radius: orbit_base * 4.3,
            orbit_speed: 0.18,
            self_speed: 1.2,
            scale: 1.7,
            has_moon: false,
        },
    ];

    // ----- Warp -----
    let mut warp = WarpState {
        active: false,
        t: 0.0,
        start_pos: cam.pos,
        end_pos: cam.pos,
    };

    let mut time_acc: f32 = 0.0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time_acc += dt;

        // Input normal sólo si no estamos en warp
        if !warp.active {
            cam.update_input(&rl, dt);
        }

        // Matrices de vista/proyección (se ajustan después de colisiones)
        let mut view = cam.view_matrix();
        let proj = cam.proj_matrix(aspect);

        // ----- Simulación de cuerpos -----
        let mut instances: Vec<Instance> = Vec::new();
        let mut collision_spheres: Vec<(glm::Vec3, f32)> = Vec::new();

        // Sol en el origen
        let sun_center_world = glm::vec3(0.0, 0.0, 0.0);
        let sun_model = glm::translation(&sun_center_world)
            * glm::rotation(time_acc * 0.20, &glm::vec3(0.0, 1.0, 0.0))
            * glm::scaling(&glm::vec3(sun_scale, sun_scale, sun_scale))
            * glm::translation(&(-sphere_center));
        let sun_collision_radius = sphere_radius * sun_scale * 1.1;

        instances.push(Instance {
            kind: BodyShader::Star,
            model: sun_model,
            center_world: sun_center_world,
            radius_collision: sun_collision_radius,
        });
        collision_spheres.push((sun_center_world, sun_collision_radius));

        // Planetas + luna de la Tierra
        let mut earth_center_world = glm::vec3(0.0, 0.0, 0.0);
        let mut jupiter_model_for_rings: Option<glm::Mat4> = None;
        let mut jupiter_radius_world = 0.0;

        for (idx, p) in planets.iter().enumerate() {
            let angle = time_acc * p.orbit_speed + idx as f32 * 0.7;
            let center_world = glm::vec3(
                p.orbit_radius * angle.cos(),
                0.0,
                p.orbit_radius * angle.sin(),
            );

            let model = glm::translation(&center_world)
                * glm::rotation(time_acc * p.self_speed, &glm::vec3(0.0, 1.0, 0.0))
                * glm::scaling(&glm::vec3(p.scale, p.scale, p.scale))
                * glm::translation(&(-sphere_center));

            let coll_r = sphere_radius * p.scale * 1.2;
            instances.push(Instance {
                kind: p.kind,
                model,
                center_world,
                radius_collision: coll_r,
            });
            collision_spheres.push((center_world, coll_r));

            if let BodyShader::Rocky = p.kind {
                earth_center_world = center_world;

                // Luna
                let moon_scale = 0.35;
                let moon_orbit_radius = sphere_radius * p.scale * 3.0;
                let moon_angle = time_acc * 1.6;

                let moon_offset = glm::vec3(
                    moon_orbit_radius * moon_angle.cos(),
                    sphere_radius * 0.3 * (moon_angle * 0.8).sin(),
                    moon_orbit_radius * moon_angle.sin(),
                );
                let moon_center_world = center_world + moon_offset;

                let moon_model = glm::translation(&moon_center_world)
                    * glm::rotation(time_acc * 1.2, &glm::vec3(0.0, 1.0, 0.0))
                    * glm::scaling(&glm::vec3(moon_scale, moon_scale, moon_scale))
                    * glm::translation(&(-sphere_center));

                let moon_coll = sphere_radius * moon_scale * 1.3;
                instances.push(Instance {
                    kind: BodyShader::Moon,
                    model: moon_model,
                    center_world: moon_center_world,
                    radius_collision: moon_coll,
                });
                collision_spheres.push((moon_center_world, moon_coll));
            }

            if let BodyShader::GasGiant = p.kind {
                jupiter_model_for_rings = Some(model);
                jupiter_radius_world = sphere_radius * p.scale;
            }
        }

        // ----- Warp: teclas a distintos puntos -----
        {
            use raylib::consts::KeyboardKey::*;
            let mut warp_target: Option<glm::Vec3> = None;

            // 1 -> vista general arriba del plano
            if rl.is_key_pressed(KEY_ONE) {
                let far = orbit_base * 6.0;
                warp_target = Some(glm::vec3(0.0, far * 0.45, far));
            }
            // 2 -> sol
            if rl.is_key_pressed(KEY_TWO) {
                let dir = glm::vec3(0.0, 0.25, 1.0).normalize();
                warp_target = Some(sun_center_world + dir * (sun_collision_radius * 4.0));
            }
            // 3 -> órbita Tierra
            if rl.is_key_pressed(KEY_THREE) {
                let dir = (earth_center_world - sun_center_world).normalize();
                let up = glm::vec3(0.0, 1.5, 0.0);
                warp_target = Some(earth_center_world - dir * (sphere_radius * 6.0) + up);
            }
            // 4 -> Marte
            if rl.is_key_pressed(KEY_FOUR) {
                if let Some(inst) = instances
                    .iter()
                    .find(|i| matches!(i.kind, BodyShader::Mars))
                {
                    let dir = (inst.center_world - sun_center_world).normalize();
                    warp_target = Some(
                        inst.center_world - dir * (inst.radius_collision * 3.0)
                            + glm::vec3(0.0, inst.radius_collision * 1.2, 0.0),
                    );
                }
            }
            // 5 -> Júpiter
            if rl.is_key_pressed(KEY_FIVE) {
                if let Some(inst) = instances
                    .iter()
                    .find(|i| matches!(i.kind, BodyShader::GasGiant))
                {
                    let dir = (inst.center_world - sun_center_world).normalize();
                    warp_target = Some(
                        inst.center_world - dir * (inst.radius_collision * 3.5)
                            + glm::vec3(0.0, inst.radius_collision * 1.4, 0.0),
                    );
                }
            }
            // 6 -> justo sobre el plano eclíptico
            if rl.is_key_pressed(KEY_SIX) {
                let far = orbit_base * 5.0;
                warp_target = Some(glm::vec3(-far, 3.0, 0.0));
            }

            if let Some(target) = warp_target {
                warp.active = true;
                warp.t = 0.0;
                warp.start_pos = cam.pos;
                warp.end_pos = target;
            }
        }

        // ----- Animación de warp (lerp con ease) -----
        if warp.active {
            warp.t += dt * 1.2;
            if warp.t >= 1.0 {
                warp.t = 1.0;
                warp.active = false;
            }
            let t = warp.t;
            let smooth = t * t * (3.0 - 2.0 * t);
            cam.pos = warp.start_pos * (1.0 - smooth) + warp.end_pos * smooth;
        }

        // ----- Colisión cámara vs. cuerpos -----
        for (center, r) in &collision_spheres {
            let delta = cam.pos - *center;
            let dist = delta.magnitude();
            let min_dist = *r * 1.05;
            if dist < min_dist {
                let dir = if dist > 1e-3 {
                    delta / dist
                } else {
                    glm::vec3(0.0, 1.0, 0.0)
                };
                cam.pos = *center + dir * min_dist;
            }
        }

        view = cam.view_matrix();

        // ----- Nave siguiendo a la cámara -----
        let ship_follow_dist = sphere_radius * 4.0;
        let ship_pos =
            cam.pos + cam.forward() * ship_follow_dist - cam.up() * (sphere_radius * 0.8);

        let ship_scale = 0.5;
        let ship_model = glm::translation(&ship_pos)
            * glm::rotation(std::f32::consts::PI, &glm::vec3(0.0, 1.0, 0.0))
            * glm::scaling(&glm::vec3(ship_scale, ship_scale, ship_scale))
            * glm::translation(&(-ship_center));

        let ship_mvp = proj * view * ship_model;
        let ship_mv = view * ship_model;

        let ship_screen_vertices: Vec<Vec3> =
            project_vertices_perspective(&ship_vertices, &ship_mvp, WIDTH, HEIGHT);

        let ship_view_vertices: Vec<glm::Vec3> = ship_vertices
            .iter()
            .map(|v| {
                let p = ship_mv * glm::vec4(v.x, v.y, v.z, 1.0);
                glm::vec3(p.x, p.y, p.z)
            })
            .collect();

        let mut ship_is_front = vec![false; ship_faces.len()];
        for (fi, f) in ship_faces.iter().enumerate() {
            let (i0, i1, i2) = (
                f.vertex_indices[0],
                f.vertex_indices[1],
                f.vertex_indices[2],
            );
            let va = ship_view_vertices[i0];
            let vb = ship_view_vertices[i1];
            let vc = ship_view_vertices[i2];
            let n = glm::normalize(&glm::cross(&(vb - va), &(vc - va)));
            ship_is_front[fi] = n.z < 0.0;
        }

        // ----- Render: limpiar framebuffer -----
        fb.clear();

        // Órbitas siempre al fondo
        for p in &planets {
            let orbit_color = Color::new(60, 90, 130, 255);
            draw_orbit(&mut fb, &view, &proj, p.orbit_radius, orbit_color);
        }

        // ----- Relleno Sol / Planetas / Lunas -----
        for inst in &instances {
            let mvp = proj * view * inst.model;
            let mv = view * inst.model;

            let screen_vertices: Vec<Vec3> =
                project_vertices_perspective(&sphere_vertices, &mvp, WIDTH, HEIGHT);

            let view_vertices: Vec<glm::Vec3> = sphere_vertices
                .iter()
                .map(|v| {
                    let p = mv * glm::vec4(v.x, v.y, v.z, 1.0);
                    glm::vec3(p.x, p.y, p.z)
                })
                .collect();

            let mut is_front = vec![false; sphere_faces.len()];
            for (fi, f) in sphere_faces.iter().enumerate() {
                let (i0, i1, i2) = (
                    f.vertex_indices[0],
                    f.vertex_indices[1],
                    f.vertex_indices[2],
                );
                let va = view_vertices[i0];
                let vb = view_vertices[i1];
                let vc = view_vertices[i2];
                let n = glm::normalize(&glm::cross(&(vb - va), &(vc - va)));
                is_front[fi] = n.z < 0.0;
            }

            for f in &sphere_faces {
                let (i0, i1, i2) = (
                    f.vertex_indices[0],
                    f.vertex_indices[1],
                    f.vertex_indices[2],
                );

                let a = screen_vertices[i0];
                let b = screen_vertices[i1];
                let c = screen_vertices[i2];

                let va_m = sphere_vertices[i0];
                let vb_m = sphere_vertices[i1];
                let vc_m = sphere_vertices[i2];
                let va_v = view_vertices[i0];
                let vb_v = view_vertices[i1];
                let vc_v = view_vertices[i2];

                let n_view = glm::normalize(&glm::cross(&(vb_v - va_v), &(vc_v - va_v)));
                let centroid = (va_m + vb_m + vc_m) / 3.0;

                let color = shade_body(
                    inst.kind,
                    centroid,
                    n_view,
                    sphere_center,
                    sphere_radius,
                    time_acc,
                );

                fb.set_color(color);
                triangle_filled(&mut fb, &a, &b, &c);
            }

            // Contorno estilo cómic
            fb.set_color(Color::BLACK);
            let thickness = 2;
            for (&(i0, i1), adj) in &sphere_edge_to_faces {
                let draw = match adj.as_slice() {
                    [f0] => is_front[*f0],
                    [f0, f1] => is_front[*f0] ^ is_front[*f1],
                    _ => false,
                };
                if draw {
                    let a = screen_vertices[i0];
                    let b = screen_vertices[i1];
                    line_depth_thick(&mut fb, &a, &b, thickness);
                }
            }
        }

        // Anillos de Júpiter (usa el modelo que calculamos arriba)
        if let Some(j_model) = jupiter_model_for_rings {
            let mvp = proj * view * j_model;
            let inner = jupiter_radius_world * 0.50;
            let outer = jupiter_radius_world * 1.00;
            draw_jupiter_rings(&mut fb, &mvp, inner, outer);
        }

        // ----- Nave Milano (relleno + contorno) -----
        for f in &ship_faces {
            let (i0, i1, i2) = (
                f.vertex_indices[0],
                f.vertex_indices[1],
                f.vertex_indices[2],
            );

            let a = ship_screen_vertices[i0];
            let b = ship_screen_vertices[i1];
            let c = ship_screen_vertices[i2];

            let va_m = ship_vertices[i0];
            let vb_m = ship_vertices[i1];
            let vc_m = ship_vertices[i2];
            let va_v = ship_view_vertices[i0];
            let vb_v = ship_view_vertices[i1];
            let vc_v = ship_view_vertices[i2];

            let n_view = glm::normalize(&glm::cross(&(vb_v - va_v), &(vc_v - va_v)));
            let base_color = choose_milano_color(va_m, vb_m, vc_m, n_view, ship_center, ship_size);

            fb.set_color(base_color);
            triangle_filled(&mut fb, &a, &b, &c);
        }

        fb.set_color(Color::BLACK);
        let ship_thickness = 2;
        for (&(i0, i1), adj) in &ship_edge_to_faces {
            let draw = match adj.as_slice() {
                [f0] => ship_is_front[*f0],
                [f0, f1] => ship_is_front[*f0] ^ ship_is_front[*f1],
                _ => false,
            };
            if draw {
                let a = ship_screen_vertices[i0];
                let b = ship_screen_vertices[i1];
                line_depth_thick(&mut fb, &a, &b, ship_thickness);
            }
        }

        // ----- Presentación (skybox + estrellas + HUD) -----
        let tex = rl
            .load_texture_from_image(&thread, &fb.color_buffer)
            .expect("No pude crear Texture2D desde el framebuffer");

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(4, 8, 20, 255));
        d.draw_texture(&tex, 0, 0, Color::WHITE);

        // Estrellas 2D estilo skybox
        for i in 0..STAR_COUNT {
            let x = ((i * 73 + 19 * i * i) % (WIDTH as usize)) as i32;
            let y = ((i * 151 + 37) % (HEIGHT as usize)) as i32;
            let b = 160 + ((i * 97) % 80) as u8;
            d.draw_pixel(x, y, Color::new(b, b, b, 255));
        }

        d.draw_text(
            "WASD/QE mover | Flechas/Mouse mirar | Z/X FOV | M mouse | 1-6 warps | P PNG",
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
