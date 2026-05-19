use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use crossterm::style::Color;
use rand::Rng;

pub struct FireVisualizer {
    buffer_data: Vec<u8>,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,
}

impl FireVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        Self {
            buffer_data: vec![0; (width as usize) * (height as usize)],
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
        }
    }

    fn get_heat(&self, x: u16, y: u16) -> u8 {
        if x < self.width && y < self.height {
            self.buffer_data[(y as usize) * (self.width as usize) + (x as usize)]
        } else {
            0
        }
    }

    fn set_heat(&mut self, x: u16, y: u16, heat: u8) {
        if x < self.width && y < self.height {
            self.buffer_data[(y as usize) * (self.width as usize) + (x as usize)] = heat;
        }
    }

    fn step_fire(&mut self) {
        let mut rng = rand::thread_rng();

        let y = self.height - 1;
        for x in 0..self.width {
            let heat = if rng.gen_bool(0.2) {
                rng.gen_range(0..=150)
            } else {
                255
            };
            self.set_heat(x, y, heat);
        }

        for y in (1..self.height).rev() {
            for x in 0..self.width {
                let heat_below = self.get_heat(x, y);

                let decay = if rng.gen_bool(0.1) {
                    0
                } else {
                    rng.gen_range(5..=35)
                };
                let new_heat = heat_below.saturating_sub(decay);

                let wind = rng.gen_range(0..=3);
                let mut new_x = x;
                if wind == 1 && x > 0 {
                    new_x -= 1;
                } else if wind == 2 && x + 1 < self.width {
                    new_x += 1;
                }

                self.set_heat(new_x, y - 1, new_heat);
            }
        }
    }
}

impl Visualizer for FireVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.accumulator += delta_time;

        let tick_rate = (1.0 / 20.0) / self.speed_multiplier;

        while self.accumulator >= tick_rate {
            self.accumulator -= tick_rate;
            self.step_fire();
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.buffer_data = vec![0; (self.width as usize) * (self.height as usize)];
        }

        let chars_len = self.charset.chars.len();
        if chars_len == 0 {
            buffer.clear();
            return;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let heat = self.get_heat(x, y);

                if heat < 10 {
                    buffer.set(x, y, ' ', Color::Reset, Color::Reset);
                    continue;
                }

                let ratio = heat as f32 / 255.0;
                let color = interpolate_gradient(&self.palette, ratio);

                let char_idx = ((ratio * chars_len as f32) as usize).min(chars_len - 1);
                let char_to_draw = self.charset.chars[char_idx];

                buffer.set(x, y, char_to_draw, color, Color::Reset);
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
