use glm::{Mat4, Vec3};
use nalgebra_glm as glm;

pub struct Camera {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_y: f32,
    pub znear: f32,
    pub zfar: f32,
    pub speed: f32,
    pub mouse_sens: f32,
    pub use_mouse: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: glm::vec3(0.0, 0.0, 3.0),
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            fov_y: glm::radians(&glm::vec1(60.0)).x,
            znear: 0.01,
            zfar: 1000.0,
            speed: 2.5,
            mouse_sens: 0.0018,
            use_mouse: false,
        }
    }
}

impl Camera {
    pub fn forward(&self) -> Vec3 {
        let cp = self.pitch.cos();
        glm::vec3(cp * self.yaw.cos(), self.pitch.sin(), cp * self.yaw.sin()).normalize()
    }
    pub fn right(&self) -> Vec3 {
        self.forward().cross(&glm::vec3(0.0, 1.0, 0.0)).normalize()
    }
    pub fn up(&self) -> Vec3 {
        self.right().cross(&self.forward()).normalize()
    }
    pub fn view_matrix(&self) -> Mat4 {
        glm::look_at(
            &self.pos,
            &(self.pos + self.forward()),
            &glm::vec3(0.0, 1.0, 0.0),
        )
    }
    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        glm::perspective(aspect, self.fov_y, self.znear, self.zfar)
    }

    pub fn update_input(&mut self, rl: &raylib::RaylibHandle, dt: f32) {
        use raylib::consts::KeyboardKey::*;
        let mut v = glm::vec3(0.0, 0.0, 0.0);
        if rl.is_key_down(KEY_W) {
            v += self.forward();
        }
        if rl.is_key_down(KEY_S) {
            v -= self.forward();
        }
        if rl.is_key_down(KEY_D) {
            v += self.right();
        }
        if rl.is_key_down(KEY_A) {
            v -= self.right();
        }
        if rl.is_key_down(KEY_E) {
            v += glm::vec3(0.0, 1.0, 0.0);
        }
        if rl.is_key_down(KEY_Q) {
            v -= glm::vec3(0.0, 1.0, 0.0);
        }
        if v.magnitude() > 1e-6 {
            self.pos += v.normalize() * self.speed * dt;
        }

        if self.use_mouse {
            let md = rl.get_mouse_delta();
            self.yaw += md.x * self.mouse_sens;
            self.pitch -= md.y * self.mouse_sens;
        } else {
            if rl.is_key_down(KEY_LEFT) {
                self.yaw -= 1.5 * dt;
            }
            if rl.is_key_down(KEY_RIGHT) {
                self.yaw += 1.5 * dt;
            }
            if rl.is_key_down(KEY_UP) {
                self.pitch += 1.5 * dt;
            }
            if rl.is_key_down(KEY_DOWN) {
                self.pitch -= 1.5 * dt;
            }
        }

        let lim = glm::radians(&glm::vec1(89.0)).x;
        if self.pitch > lim {
            self.pitch = lim;
        }
        if self.pitch < -lim {
            self.pitch = -lim;
        }

        if rl.is_key_down(KEY_Z) {
            self.fov_y -= 1.0 * dt;
        }
        if rl.is_key_down(KEY_X) {
            self.fov_y += 1.0 * dt;
        }
        self.fov_y = self.fov_y.clamp(
            glm::radians(&glm::vec1(20.0)).x,
            glm::radians(&glm::vec1(100.0)).x,
        );

        if rl.is_key_pressed(KEY_M) {
            self.use_mouse = !self.use_mouse;
        }
    }
}
