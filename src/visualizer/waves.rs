use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::utils::math3d::Vec3;
use crate::visualizer::Visualizer;
use noise::{NoiseFn, Perlin};

pub struct WavesVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,
    z_offset: f64,
    perlin: Perlin,
}

impl WavesVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        Self {
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
            z_offset: 35.0,
            perlin: Perlin::new(1337),
        }
    }
}

impl Visualizer for WavesVisualizer {
    fn update(&mut self, mut delta_time: f64) {
        delta_time *= self.speed_multiplier;
        self.accumulator += delta_time;
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

        let time = self.accumulator * 2.0;

        let grid_size = 140;
        let spacing = 0.5;

        for grid_z in 0..grid_size {
            for grid_x in 0..grid_size {
                let x = (grid_x as f64 - grid_size as f64 / 2.0) * spacing;
                let z = (grid_z as f64 - grid_size as f64 / 2.0) * spacing;

                let nx = x * 0.15;
                let nz = z * 0.15;

                let n1 = self.perlin.get([nx + time * 0.4, nz, time * 0.2]);
                let n2 =
                    self.perlin
                        .get([nx * 2.1 - time * 0.5, nz * 2.1 + time * 0.3, time * 0.4])
                        * 0.5;
                let n3 = self
                    .perlin
                    .get([nx * 4.3, nz * 4.3 - time * 0.7, time * 0.6])
                    * 0.25;

                let y = (n1 + n2 + n3) * 2.0 - 2.0;

                let point = Vec3::new(x, y, z);

                let rotated = point.rotate_x(0.4);

                if let Some((sx, sy)) =
                    rotated.project_to_2d(self.width as f64, self.height as f64, 2.0, self.z_offset)
                {
                    if sx >= 0.0 && sx < self.width as f64 && sy >= 0.0 && sy < self.height as f64 {
                        let normalized = ((y + 4.0) / 4.0).clamp(0.0, 1.0);

                        let color = interpolate_gradient(&self.palette, normalized as f32);
                        let char_idx =
                            ((normalized * chars_len as f64) as usize).min(chars_len - 1);
                        let char_to_draw = self.charset.chars[char_idx];

                        buffer.set(
                            sx as u16,
                            sy as u16,
                            char_to_draw,
                            color,
                            crossterm::style::Color::Reset,
                        );
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
        self.z_offset = self.z_offset.clamp(10.0, 60.0);
    }
}
