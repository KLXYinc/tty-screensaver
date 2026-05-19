use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

struct Bubble {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
    color_offset: f32,
}

pub struct BubblesVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,
    bubbles: Vec<Bubble>,
}

impl BubblesVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
            bubbles: Vec::new(),
        };
        vis.init_bubbles();
        vis
    }

    fn init_bubbles(&mut self) {
        let mut rng = rand::thread_rng();
        self.bubbles.clear();
        let num_bubbles = rng.gen_range(5..15);

        for _ in 0..num_bubbles {
            self.bubbles.push(Bubble {
                x: rng.gen_range(10.0..(self.width as f32 - 10.0).max(11.0)),
                y: rng.gen_range(5.0..(self.height as f32 - 5.0).max(6.0)),
                vx: rng.gen_range(5.0..20.0) * if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
                vy: rng.gen_range(2.0..10.0) * if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
                radius: rng.gen_range(2.0..8.0),
                color_offset: rng.gen_range(0.0..1.0),
            });
        }
    }
}

impl Visualizer for BubblesVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = (delta_time * self.speed_multiplier) as f32;

        for b in &mut self.bubbles {
            b.x += b.vx * dt;
            b.y += b.vy * dt;

            let rx = b.radius * 2.0;
            let ry = b.radius;

            if b.x - rx < 0.0 {
                b.x = rx;
                b.vx *= -1.0;
            } else if b.x + rx >= self.width as f32 {
                b.x = self.width as f32 - rx - 1.0;
                b.vx *= -1.0;
            }

            if b.y - ry < 0.0 {
                b.y = ry;
                b.vy *= -1.0;
            } else if b.y + ry >= self.height as f32 {
                b.y = self.height as f32 - ry - 1.0;
                b.vy *= -1.0;
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.init_bubbles();
        }
        buffer.clear();

        let chars_len = self.charset.chars.len();
        if chars_len == 0 {
            return;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let mut best_intensity = 0.0;
                let mut best_color_offset = 0.0;

                for b in &self.bubbles {
                    let dx = (x as f32 - b.x) * 0.5;
                    let dy = y as f32 - b.y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    let ring_dist = (dist - b.radius).abs();

                    if ring_dist < 1.0 {
                        let intensity = 1.0 - ring_dist;
                        if intensity > best_intensity {
                            best_intensity = intensity;
                            best_color_offset = b.color_offset;
                        }
                    }

                    let hx = (x as f32 - (b.x + b.radius * 0.8)) * 0.5;
                    let hy = y as f32 - (b.y - b.radius * 0.5);
                    let hdist = (hx * hx + hy * hy).sqrt();
                    if hdist < 0.8 {
                        if 1.5 > best_intensity {
                            best_intensity = 1.5;
                            best_color_offset = 1.0;
                        }
                    }
                }

                if best_intensity > 0.1 {
                    let clamped = best_intensity.min(1.0);
                    let color = interpolate_gradient(&self.palette, best_color_offset * clamped);

                    let char_idx =
                        ((clamped * (chars_len as f32 - 1.0)) as usize).min(chars_len - 1);
                    let char_to_draw = self.charset.chars[char_idx];

                    buffer.set(x, y, char_to_draw, color, crossterm::style::Color::Reset);
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
        self.speed_multiplier = self.speed_multiplier.clamp(0.01, 10000.0);
    }
}
