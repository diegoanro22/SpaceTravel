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

use crate::camera::Camera;
use crate::framebuffer::FrameBuffer;
use crate::geom::Vec3;
use crate::obj::load_obj;
use crate::transform::project_vertices_perspective;
use crate::triangle::triangle;

const WIDTH: i32 = 1000;
const HEIGHT: i32 = 700;

fn main() -> anyhow::Result<()> {
    // Init ventana
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Software Renderer – Wireframe OBJ + Cámara")
        .build();
    rl.set_target_fps(120);

    let mut fb = FrameBuffer::new(WIDTH, HEIGHT, Color::BLACK);

    ///////////////////
    // Cargar modelo //
    ///////////////////

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/Nave_espacial.obj".to_string());
    let (vertices, faces) = load_obj(&path)?;
    println!(
        "Vértices: {} | Caras (triángulos): {}",
        vertices.len(),
        faces.len()
    );

    // Bounding box y centro del modelo
    let mut min = glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = glm::vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for v in &vertices {
        min.x = min.x.min(v.x);
        min.y = min.y.min(v.y);
        min.z = min.z.min(v.z);
        max.x = max.x.max(v.x);
        max.y = max.y.max(v.y);
        max.z = max.z.max(v.z);
    }
    let center = (min + max) * 0.5;

    // Radio de la esfera envolvente
    let size = max - min;
    let r = 0.5 * (size.x * size.x + size.y * size.y + size.z * size.z).sqrt();

    // Cámara
    let mut cam = Camera::default();
    let aspect = (WIDTH as f32) / (HEIGHT as f32);

    // Distancia para que quepa el modelo con margen
    let mut dist = if (cam.fov_y * 0.5).tan() > 1e-6 {
        r / (cam.fov_y * 0.5).tan()
    } else {
        r + 1.0
    };
    dist *= 1.2; // margen
    cam.pos = glm::vec3(0.0, 0.0, dist);
    cam.zfar = (dist + r * 2.0).max(1000.0);

    // Loop
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // Input cámara
        cam.update_input(&rl, dt);

        // Matrices
        let model = glm::translation(&(-center));
        let view = cam.view_matrix();
        let proj = cam.proj_matrix(aspect);
        let mvp = proj * view * model;

        // Proyectar a coordenadas de pantalla
        let screen_vertices: Vec<Vec3> =
            project_vertices_perspective(&vertices, &mvp, WIDTH, HEIGHT);

        // Dibujar wireframe al framebuffer
        fb.clear();
        fb.set_color(Color::YELLOW);
        for f in &faces {
            let a = screen_vertices[f.vertex_indices[0]];
            let b = screen_vertices[f.vertex_indices[1]];
            let c = screen_vertices[f.vertex_indices[2]];
            triangle(&mut fb, &a, &b, &c);
        }

        let tex = rl
            .load_texture_from_image(&thread, &fb.color_buffer)
            .expect("No pude crear Texture2D desde el framebuffer");

        // Presentar
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex, 0, 0, Color::WHITE);

        d.draw_text(
            "WASD/QE: mover | Mouse/Flechas: mirar | Z/X: FOV | M: mouse on/off | P: PNG",
            10,
            10,
            16,
            Color::RAYWHITE,
        );
        d.draw_text(
            &format!(
                "pos=({:.2},{:.2},{:.2}) yaw={:.2} pitch={:.2}",
                cam.pos.x, cam.pos.y, cam.pos.z, cam.yaw, cam.pitch
            ),
            10,
            30,
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
