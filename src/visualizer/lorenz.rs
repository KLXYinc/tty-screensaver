use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

#[derive(Clone, Copy)]
struct Particle {
    x: f64,
    y: f64,
    z: f64,
}

pub struct LorenzVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    particles: Vec<Particle>,
    sigma: f64,
    rho: f64,
    beta: f64,
    speed: f64,
    zoom: f64,
    time: f64,
}

impl LorenzVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut rng = rand::thread_rng();
        let num_particles = 150;
        let mut particles = Vec::with_capacity(num_particles);

        let start_x = 0.1;
        let start_y = 0.0;
        let start_z = 20.0;

        for _ in 0..num_particles {
            particles.push(Particle {
                x: start_x + rng.gen_range(-0.01..0.01),
                y: start_y + rng.gen_range(-0.01..0.01),
                z: start_z + rng.gen_range(-0.01..0.01),
            });
        }

        Self {
            width,
            height,
            palette,
            charset,
            particles,
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
            speed: 1.0,
            zoom: 1.0,
            time: 0.0,
        }
    }
}

impl Visualizer for LorenzVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = delta_time * self.speed;
        self.time += dt;
        let steps = 4;
        let sub_dt = dt / steps as f64;

        for _ in 0..steps {
            for p in &mut self.particles {
                let dx = self.sigma * (p.y - p.x);
                let dy = p.x * (self.rho - p.z) - p.y;
                let dz = p.x * p.y - self.beta * p.z;

                p.x += dx * sub_dt;
                p.y += dy * sub_dt;
                p.z += dz * sub_dt;

                if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
                    p.x = 0.1;
                    p.y = 0.0;
                    p.z = 20.0;
                }
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
        }

        buffer.clear();
        if self.charset.chars.is_empty() {
            return;
        }

        let angle_y = self.time * 0.3;
        let angle_x = self.time * 0.15;
        let center_z = 25.0;
        let scale = (self.height as f64) / 55.0 * self.zoom;

        for p in &self.particles {
            let tx = p.x;
            let ty = p.y;
            let tz = p.z - center_z;

            let x1 = tx * angle_y.cos() - tz * angle_y.sin();
            let z1 = tx * angle_y.sin() + tz * angle_y.cos();

            let _y2 = ty * angle_x.cos() - z1 * angle_x.sin();
            let z2 = ty * angle_x.sin() + z1 * angle_x.cos();
            let x2 = x1;

            let screen_x = ((self.width as f64 / 2.0) + x2 * scale * 2.0).round() as i32;
            let screen_y = ((self.height as f64 / 2.0) - z2 * scale).round() as i32;

            if screen_x >= 0
                && screen_x < self.width as i32
                && screen_y >= 0
                && screen_y < self.height as i32
            {
                let normalized_z = ((z2 + 30.0) / 60.0).clamp(0.0, 1.0);
                let color = interpolate_gradient(&self.palette, normalized_z as f32);

                let char_idx =
                    (normalized_z * (self.charset.chars.len() as f64 - 1.0)).round() as usize;
                let c = self.charset.chars[char_idx.clamp(0, self.charset.chars.len() - 1)];

                buffer.set(
                    screen_x as u16,
                    screen_y as u16,
                    c,
                    color,
                    crossterm::style::Color::Reset,
                );
            }
        }
    }

    fn set_palette(&mut self, palette: ThemePalette) {
        self.palette = palette;
    }
    fn set_charset(&mut self, charset: CharSet) {
        self.charset = charset;
    }

    fn on_scroll(&mut self, delta: i32) {
        self.speed += delta as f64 * 0.2;
        self.speed = self.speed.clamp(0.1, 10.0);
    }

    fn on_scroll_ext(&mut self, delta: i32, is_ctrl: bool) {
        if is_ctrl {
            self.zoom += delta as f64 * 0.1;
            self.zoom = self.zoom.clamp(0.1, 5.0);
        } else {
            self.on_scroll(delta);
        }
    }
}
