use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use noise::{NoiseFn, Perlin};

pub struct PerlinVisualizer {
    perlin: Perlin,
    z_offset: f64,
    palette: ThemePalette,
    charset: CharSet,
    speed_multiplier: f64,
}

impl PerlinVisualizer {
    pub fn new(_width: u16, _height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        Self {
            perlin: Perlin::new(1),
            z_offset: 0.0,
            palette,
            charset,
            speed_multiplier: 1.0,
        }
    }
}

impl Visualizer for PerlinVisualizer {
    fn update(&mut self, mut delta_time: f64) {
        delta_time *= self.speed_multiplier;
        self.z_offset += 0.5 * delta_time;
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        let chars_len = self.charset.chars.len();
        if chars_len == 0 {
            buffer.clear();
            return;
        }

        let scale = 0.05;

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let val =
                    self.perlin
                        .get([x as f64 * scale, y as f64 * scale * 2.0, self.z_offset]);

                let normalized = (val + 1.0) / 2.0;

                let char_idx = ((normalized * chars_len as f64) as usize).min(chars_len - 1);
                let char_to_draw = self.charset.chars[char_idx];

                if normalized > 0.1 {
                    let color = interpolate_gradient(&self.palette, normalized as f32);
                    buffer.set(x, y, char_to_draw, color, crossterm::style::Color::Reset);
                } else {
                    buffer.set(
                        x,
                        y,
                        ' ',
                        crossterm::style::Color::Reset,
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
        self.speed_multiplier = self.speed_multiplier.clamp(0.01, 10000.0);
    }
}
