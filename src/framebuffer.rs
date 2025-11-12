use raylib::prelude::*;

pub struct FrameBuffer {
    pub width: i32,
    pub height: i32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
    z_buffer: Vec<f32>, // <-- NUEVO
}

impl FrameBuffer {
    pub fn new(width: i32, height: i32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width, height, background_color);
        FrameBuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
            z_buffer: vec![f32::INFINITY; (width * height) as usize], // <-- NUEVO
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
        self.color_buffer = Image::gen_image_color(self.width, self.height, color);
        self.z_buffer.fill(f32::INFINITY);
    }

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(self.width, self.height, self.background_color);
        self.z_buffer.fill(f32::INFINITY); // <-- NUEVO
    }

    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    #[inline]
    fn in_bounds(&self, x: i32, y: i32) -> bool {
        (0..self.width).contains(&x) && (0..self.height).contains(&y)
    }
    #[inline]
    fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32) {
        if self.in_bounds(x, y) {
            self.color_buffer.draw_pixel(x, y, self.current_color);
        }
    }

    // <-- NUEVO: píxel con prueba de profundidad
    #[inline]
    pub fn set_pixel_z(&mut self, x: i32, y: i32, z: f32) {
        if self.in_bounds(x, y) {
            let i = self.idx(x, y);
            if z < self.z_buffer[i] {
                self.z_buffer[i] = z;
                self.color_buffer.draw_pixel(x, y, self.current_color);
            }
        }
    }

    pub fn render_to_file(&self, file_path: &str) -> anyhow::Result<()> {
        self.color_buffer.export_image(file_path);
        Ok(())
    }
}
