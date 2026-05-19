use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

struct Blob {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    radius: f64,
}

pub struct MetaballsVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    blobs: Vec<Blob>,
    speed_multiplier: f64,
    zoom: f64,
}

impl MetaballsVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut rng = rand::thread_rng();
        let num_blobs = 10;
        let mut blobs = Vec::with_capacity(num_blobs);

        for _ in 0..num_blobs {
            blobs.push(Blob {
                x: rng.gen_range(0.0..width as f64),
                y: rng.gen_range(0.0..height as f64),
                vx: rng.gen_range(-20.0..20.0),
                vy: rng.gen_range(-20.0..20.0),
                radius: rng.gen_range(8.0..18.0),
            });
        }

        Self {
            width,
            height,
            palette,
            charset,
            blobs,
            speed_multiplier: 1.0,
            zoom: 1.0,
        }
    }
}

impl Visualizer for MetaballsVisualizer {
    fn update(&mut self, delta_time: f64) {
        let w = self.width as f64;
        let h = self.height as f64;
        let dt = delta_time * self.speed_multiplier;

        for blob in &mut self.blobs {
            blob.x += blob.vx * dt;
            blob.y += blob.vy * dt;

            if blob.x < blob.radius {
                blob.x = blob.radius;
                blob.vx *= -1.0;
            } else if blob.x > w - blob.radius {
                blob.x = w - blob.radius;
                blob.vx *= -1.0;
            }

            if blob.y < blob.radius {
                blob.y = blob.radius;
                blob.vy *= -1.0;
            } else if blob.y > h - blob.radius {
                blob.y = h - blob.radius;
                blob.vy *= -1.0;
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

        let threshold = 1.0;

        for y in 0..self.height {
            for x in 0..self.width {
                let mut sum = 0.0;

                let px = x as f64;
                let py = (y as f64) * 2.0;

                let cx = self.width as f64 / 2.0;
                let cy = self.height as f64;

                for blob in &self.blobs {
                    let bx = cx + (blob.x - cx) * self.zoom;
                    let by = cy + (blob.y * 2.0 - cy) * self.zoom;
                    let r = blob.radius * self.zoom;

                    let dist_sq = (px - bx).powi(2) + (py - by).powi(2);
                    if dist_sq > 0.01 {
                        sum += (r * r) / dist_sq;
                    }
                }

                if sum > threshold * 0.4 {
                    let len = self.charset.chars.len() as f64;
                    let normalized_sum = ((sum - 0.4) / 0.8).clamp(0.0, 1.0);
                    let idx = (normalized_sum * (len - 1.0)).round() as usize;
                    let char_to_draw =
                        self.charset.chars[idx.clamp(0, self.charset.chars.len() - 1)];

                    let color = interpolate_gradient(&self.palette, normalized_sum as f32);

                    buffer.set(
                        x as u16,
                        y as u16,
                        char_to_draw,
                        color,
                        crossterm::style::Color::Reset,
                    );
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
        self.speed_multiplier += delta as f64 * 0.2;
        self.speed_multiplier = self.speed_multiplier.clamp(0.1, 10.0);
    }

    fn on_scroll_ext(&mut self, delta: i32, is_ctrl: bool) {
        if is_ctrl {
            self.zoom += delta as f64 * 0.1;
            self.zoom = self.zoom.clamp(0.2, 5.0);
        } else {
            self.on_scroll(delta);
        }
    }
}
