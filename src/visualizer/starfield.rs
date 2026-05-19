use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::utils::math3d::Vec3;
use crate::visualizer::Visualizer;
use rand::Rng;

pub struct StarfieldVisualizer {
    stars: Vec<Vec3>,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    max_z: f64,
    camera_angle_z: f64,
    camera_angle_y: f64,
    speed_multiplier: f64,
}

impl StarfieldVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut visualizer = Self {
            stars: Vec::new(),
            width,
            height,
            palette,
            charset,
            max_z: 100.0,
            camera_angle_z: 0.0,
            camera_angle_y: 0.0,
            speed_multiplier: 1.0,
        };
        visualizer.init_stars();
        visualizer
    }

    fn init_stars(&mut self) {
        self.stars.clear();
        let mut rng = rand::thread_rng();
        let num_stars = (self.width as usize * self.height as usize) / 8;

        for _ in 0..num_stars {
            let angle = rng.gen_range(0.0..std::f64::consts::TAU);
            let radius = rng.gen_range(0.8..3.0);

            self.stars.push(Vec3::new(
                angle.cos() * radius,
                angle.sin() * radius,
                rng.gen_range(1.0..self.max_z),
            ));
        }
    }
}

impl Visualizer for StarfieldVisualizer {
    fn update(&mut self, mut delta_time: f64) {
        delta_time *= self.speed_multiplier;

        let mut rng = rand::thread_rng();
        let speed = 25.0;

        self.camera_angle_z += 0.2 * delta_time;
        self.camera_angle_y = (self.camera_angle_z * 0.5).sin() * 0.1;

        for star in &mut self.stars {
            star.z -= speed * delta_time;

            if star.z <= 0.1 {
                let angle = rng.gen_range(0.0..std::f64::consts::TAU);
                let radius = rng.gen_range(0.8..3.0);

                star.x = angle.cos() * radius;
                star.y = angle.sin() * radius;
                star.z = self.max_z;
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.init_stars();
        }

        buffer.clear();

        if self.charset.chars.is_empty() {
            return;
        }

        let chars_len = self.charset.chars.len();

        let time = self.camera_angle_z * 5.0;
        let tunnel_dx = (time * 0.3).sin() * 2.0 + (time * 0.77).cos() * 1.5;
        let tunnel_dy = (time * 0.43).cos() * 2.0 + (time * 0.61).sin() * 1.5;

        for star in &mut self.stars {
            let mut rotated = star
                .rotate_z(self.camera_angle_z)
                .rotate_y(self.camera_angle_y);

            let curve_factor = (rotated.z / self.max_z).powf(1.5);
            rotated.x += tunnel_dx * curve_factor;
            rotated.y += tunnel_dy * curve_factor;

            let mut on_screen = false;

            if let Some((sx, sy)) =
                rotated.project_to_2d(self.width as f64, self.height as f64, 2.5, 0.0)
            {
                if sx >= 0.0 && sx < self.width as f64 && sy >= 0.0 && sy < self.height as f64 {
                    on_screen = true;
                    let x = sx as u16;
                    let y = sy as u16;

                    let ratio = 1.0 - (star.z / self.max_z).clamp(0.0, 1.0) as f32;

                    if ratio < 0.1 {
                        buffer.set(
                            x,
                            y,
                            ' ',
                            crossterm::style::Color::Reset,
                            crossterm::style::Color::Reset,
                        );
                    } else {
                        let color = interpolate_gradient(&self.palette, ratio);
                        let char_idx = ((ratio * chars_len as f32) as usize).min(chars_len - 1);
                        let char_to_draw = self.charset.chars[char_idx];
                        buffer.set(x, y, char_to_draw, color, crossterm::style::Color::Reset);
                    }
                }
            }

            if !on_screen && star.z < self.max_z - 5.0 {
                let mut rng = rand::thread_rng();
                let angle = rng.gen_range(0.0..std::f64::consts::TAU);
                let radius = rng.gen_range(0.8..3.0);

                star.z = self.max_z;
                star.x = angle.cos() * radius;
                star.y = angle.sin() * radius;
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
        self.speed_multiplier += delta as f64 * 0.2;
        self.speed_multiplier = self.speed_multiplier.clamp(0.01, 10000.0);
    }
}
