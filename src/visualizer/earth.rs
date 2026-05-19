use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::utils::math3d::Vec3;
use crate::visualizer::Visualizer;
use noise::{NoiseFn, Perlin};
use std::f64::consts::PI;

pub struct EarthVisualizer {
    points: Vec<(Vec3, f64)>,
    angle_y: f64,
    angle_x: f64,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    speed_multiplier: f64,
    z_offset: f64,
}

impl EarthVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            points: Vec::new(),
            angle_y: 0.0,
            angle_x: 0.2,
            width,
            height,
            palette,
            charset,
            speed_multiplier: 1.0,
            z_offset: 12.0,
        };
        vis.init_sphere();
        vis
    }

    fn init_sphere(&mut self) {
        self.points.clear();

        let radius = 2.0;
        let num_points = 8000;
        let perlin = Perlin::new(42);

        let phi = PI * (3.0 - (5.0f64).sqrt());

        for i in 0..num_points {
            let y = 1.0 - (i as f64 / (num_points as f64 - 1.0)) * 2.0;
            let radius_at_y = (1.0 - y * y).sqrt();

            let theta = phi * i as f64;

            let x = theta.cos() * radius_at_y;
            let z = theta.sin() * radius_at_y;

            let vec = Vec3::new(x * radius, y * radius, z * radius);

            let n1 = perlin.get([vec.x * 0.5, vec.y * 0.5, vec.z * 0.5]);
            let n2 = perlin.get([vec.x * 1.0, vec.y * 1.0, vec.z * 1.0]) * 0.5;
            let n3 = perlin.get([vec.x * 2.0, vec.y * 2.0, vec.z * 2.0]) * 0.25;
            let val = n1 + n2 + n3;

            self.points.push((vec, val));
        }
    }
}

impl Visualizer for EarthVisualizer {
    fn update(&mut self, mut delta_time: f64) {
        delta_time *= self.speed_multiplier;
        self.angle_y -= 0.8 * delta_time;
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
        }

        buffer.clear();

        let chars_len = self.charset.chars.len();
        if chars_len == 0 {
            return;
        }

        for (point, noise_val) in &self.points {
            let rotated = point.rotate_x(self.angle_x).rotate_y(self.angle_y);

            if rotated.z > 0.0 {
                if let Some((sx, sy)) =
                    rotated.project_to_2d(self.width as f64, self.height as f64, 2.0, self.z_offset)
                {
                    if sx >= 0.0 && sx < self.width as f64 && sy >= 0.0 && sy < self.height as f64 {
                        let is_land = *noise_val > 0.05;

                        if is_land {
                            let normalized_topo = ((*noise_val - 0.05) * 1.5).clamp(0.1, 1.0);
                            let color = interpolate_gradient(&self.palette, normalized_topo as f32);

                            let char_idx =
                                ((normalized_topo * chars_len as f64) as usize).min(chars_len - 1);
                            let char_to_draw = self.charset.chars[char_idx];

                            buffer.set(
                                sx as u16,
                                sy as u16,
                                char_to_draw,
                                color,
                                crossterm::style::Color::Reset,
                            );
                        } else {
                            let color = interpolate_gradient(&self.palette, 0.0);
                            buffer.set(
                                sx as u16,
                                sy as u16,
                                '·',
                                color,
                                crossterm::style::Color::Reset,
                            );
                        }
                    }
                }
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
        self.z_offset -= delta as f64 * 0.5;
        self.z_offset = self.z_offset.clamp(3.0, 20.0);
    }
}
