use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

#[derive(Clone, Copy)]
struct Boid {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

pub struct BoidsVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    boids: Vec<Boid>,
    speed_multiplier: f64,
}

impl BoidsVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut rng = rand::thread_rng();
        let num_boids = 150;
        let mut boids = Vec::with_capacity(num_boids);

        for _ in 0..num_boids {
            boids.push(Boid {
                x: rng.gen_range(0.0..width as f64),
                y: rng.gen_range(0.0..height as f64),
                vx: rng.gen_range(-15.0..15.0),
                vy: rng.gen_range(-15.0..15.0),
            });
        }

        Self {
            width,
            height,
            palette,
            charset,
            boids,
            speed_multiplier: 1.0,
        }
    }

    fn distance(b1: &Boid, b2: &Boid) -> f64 {
        ((b1.x - b2.x).powi(2) + (b1.y - b2.y).powi(2)).sqrt()
    }
}

impl Visualizer for BoidsVisualizer {
    fn update(&mut self, delta_time: f64) {
        let visual_range = 15.0;
        let centering_factor = 0.01;
        let avoid_factor = 0.05;
        let matching_factor = 0.05;
        let min_speed = 10.0;
        let max_speed = 30.0;
        let turn_factor = 15.0;
        let margin = 5.0;

        let dt = delta_time * self.speed_multiplier;

        let num_boids = self.boids.len();
        let mut new_boids = self.boids.clone();

        for i in 0..num_boids {
            let mut center_x = 0.0;
            let mut center_y = 0.0;
            let mut close_dx = 0.0;
            let mut close_dy = 0.0;
            let mut avg_vx = 0.0;
            let mut avg_vy = 0.0;
            let mut neighbors = 0;

            let boid = &self.boids[i];

            for j in 0..num_boids {
                if i == j {
                    continue;
                }
                let other = &self.boids[j];
                let dist = Self::distance(boid, other);

                if dist < visual_range {
                    center_x += other.x;
                    center_y += other.y;
                    avg_vx += other.vx;
                    avg_vy += other.vy;
                    neighbors += 1;

                    if dist < 3.0 {
                        close_dx += boid.x - other.x;
                        close_dy += boid.y - other.y;
                    }
                }
            }

            let mut new_vx = boid.vx;
            let mut new_vy = boid.vy;

            if neighbors > 0 {
                center_x /= neighbors as f64;
                center_y /= neighbors as f64;
                avg_vx /= neighbors as f64;
                avg_vy /= neighbors as f64;

                new_vx += (center_x - boid.x) * centering_factor;
                new_vy += (center_y - boid.y) * centering_factor;

                new_vx += (avg_vx - boid.vx) * matching_factor;
                new_vy += (avg_vy - boid.vy) * matching_factor;
            }

            new_vx += close_dx * avoid_factor;
            new_vy += close_dy * avoid_factor;

            if boid.x < margin {
                new_vx += turn_factor * dt;
            }
            if boid.x > self.width as f64 - margin {
                new_vx -= turn_factor * dt;
            }
            if boid.y < margin {
                new_vy += turn_factor * dt * 2.0;
            }
            if boid.y > self.height as f64 - margin {
                new_vy -= turn_factor * dt * 2.0;
            }

            let speed = (new_vx.powi(2) + new_vy.powi(2)).sqrt();
            if speed < min_speed {
                new_vx = (new_vx / speed) * min_speed;
                new_vy = (new_vy / speed) * min_speed;
            }
            if speed > max_speed {
                new_vx = (new_vx / speed) * max_speed;
                new_vy = (new_vy / speed) * max_speed;
            }

            new_boids[i].vx = new_vx;
            new_boids[i].vy = new_vy;
            new_boids[i].x += new_vx * dt;
            new_boids[i].y += new_vy * dt;
        }

        self.boids = new_boids;
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

        for boid in &self.boids {
            let px = boid.x as i32;
            let py = boid.y as i32;

            if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                let angle = boid.vy.atan2(boid.vx);
                let normalized_angle =
                    (angle + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
                let color = interpolate_gradient(&self.palette, normalized_angle as f32);

                let char_idx = if boid.vx.abs() > boid.vy.abs() {
                    if boid.vx > 0.0 {
                        0
                    } else {
                        2 % self.charset.chars.len()
                    }
                } else {
                    if boid.vy > 0.0 {
                        1 % self.charset.chars.len()
                    } else {
                        3 % self.charset.chars.len()
                    }
                };
                let char_to_draw =
                    self.charset.chars[char_idx.clamp(0, self.charset.chars.len() - 1)];

                buffer.set(
                    px as u16,
                    py as u16,
                    char_to_draw,
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
        self.speed_multiplier += delta as f64 * 0.2;
        self.speed_multiplier = self.speed_multiplier.clamp(0.1, 10.0);
    }
}
