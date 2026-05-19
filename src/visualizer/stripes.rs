use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

struct Streak {
    freq: f32,
    amp: f32,
    phase: f32,
    phase_speed: f32,
    y_offset: f32,
    thickness: f32,
    wobble_freq: f32,
}

pub struct StripesVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,
    streaks: Vec<Streak>,
}

impl StripesVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
            streaks: Vec::new(),
        };
        vis.init_streaks();
        vis
    }

    fn init_streaks(&mut self) {
        let mut rng = rand::thread_rng();
        self.streaks.clear();
        let num_streaks = rng.gen_range(4..8);

        for i in 0..num_streaks {
            let y_rel = (i as f32 / num_streaks as f32) + rng.gen_range(-0.2..0.2);
            self.streaks.push(Streak {
                freq: rng.gen_range(0.02..0.08),
                amp: rng.gen_range(0.1..0.4),
                phase: rng.gen_range(0.0..std::f32::consts::TAU),
                phase_speed: rng.gen_range(0.5..2.5) * if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
                y_offset: y_rel,
                thickness: rng.gen_range(1.0..4.0),
                wobble_freq: rng.gen_range(0.01..0.05),
            });
        }
    }
}

impl Visualizer for StripesVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = delta_time * self.speed_multiplier;

        for s in &mut self.streaks {
            s.phase += s.phase_speed * dt as f32;
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.init_streaks();
        }
        buffer.clear();

        let chars_len = self.charset.chars.len();
        if chars_len == 0 {
            return;
        }

        for y in 0..self.height {
            let ny = y as f32 / self.height as f32;
            for x in 0..self.width {
                let nx = x as f32 / self.width as f32;

                let mut intensity = 0.0;

                for s in &self.streaks {
                    let wobble = (nx * 10.0 + s.phase).sin() * 0.05;
                    let curve_y =
                        s.y_offset + ((x as f32 * s.freq + s.phase).sin() * s.amp) + wobble;

                    let phys_dist = ((ny - curve_y) * self.height as f32).abs();

                    intensity += (s.thickness / (phys_dist + 0.1)).powf(1.1);
                }

                if intensity > 0.1 {
                    let clamped = intensity.min(1.0);
                    let color = interpolate_gradient(&self.palette, clamped);
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
