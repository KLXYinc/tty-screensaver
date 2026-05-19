use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;

pub struct SynthwaveVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    time: f64,
    speed_multiplier: f64,
}

impl SynthwaveVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        Self {
            width,
            height,
            palette,
            charset,
            time: 0.0,
            speed_multiplier: 1.0,
        }
    }
}

impl Visualizer for SynthwaveVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.time += delta_time * self.speed_multiplier;
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

        let center_x = self.width as f32 / 2.0;
        let horizon_y = (self.height as f32 * 0.5).round() as i32;

        let grid_size = 8.0;
        let focal_length = 35.0;
        let camera_height = 100.0;

        let sun_r = self.height as f32 * 0.25;
        let sun_cy = horizon_y as f32 - sun_r * 0.4;

        for y in 0..horizon_y {
            let _drawn_any = false;
            for x in 0..self.width {
                let mnt_x = (x as f32) + (self.time as f32 * 0.5);
                let mut mnt_height = 0.0;
                mnt_height += (mnt_x * 0.04).sin() * 5.0;
                mnt_height += (mnt_x * 0.09).sin() * 3.0;
                mnt_height += (mnt_x * 0.17).cos() * 1.5;
                mnt_height *= 1.5;
                mnt_height = mnt_height.max(0.0);

                let dx_from_center = (x as f32 - center_x).abs();
                let valley_width = self.width as f32 * 0.25;
                let valley_factor = (dx_from_center / valley_width).powf(1.5).clamp(0.0, 1.0);
                mnt_height *= valley_factor;

                if y as f32 > horizon_y as f32 - mnt_height {
                    let edge_dist = (y as f32 - (horizon_y as f32 - mnt_height)).abs();
                    if edge_dist < 1.0 {
                        let color = interpolate_gradient(&self.palette, 0.7);
                        buffer.set(x, y as u16, '▒', color, crossterm::style::Color::Reset);
                    } else {
                        let color = interpolate_gradient(&self.palette, 0.1);
                        buffer.set(x, y as u16, '░', color, crossterm::style::Color::Reset);
                    }
                    continue;
                }

                let dy_sun = y as f32 - sun_cy;
                let dx_sun = (x as f32 - center_x) * 0.5;
                let dist_sq = dx_sun * dx_sun + dy_sun * dy_sun;

                if dist_sq < sun_r * sun_r {
                    let sun_y_norm = (y as f32 - (sun_cy - sun_r)) / (sun_r * 2.0);

                    let mut is_cutout = false;
                    if sun_y_norm > 0.5 {
                        let lines_from_horizon = horizon_y - y as i32;
                        if matches!(lines_from_horizon, 1 | 3 | 6 | 10) {
                            is_cutout = true;
                        }
                    }

                    if !is_cutout {
                        let color =
                            interpolate_gradient(&self.palette, 1.0 - sun_y_norm.clamp(0.0, 1.0));
                        buffer.set(x, y as u16, '█', color, crossterm::style::Color::Reset);
                    }
                }
            }
        }

        for y in horizon_y..self.height as i32 {
            let dy = (y - horizon_y) as f32;
            if dy <= 0.0 {
                continue;
            }

            let z = camera_height / dy;
            if z > 100.0 {
                continue;
            }

            let scrolled_z = z + (self.time as f32 * 15.0);

            let intensity = (1.0 - (z / 50.0)).clamp(0.0, 1.0);
            if intensity <= 0.01 {
                continue;
            }

            let thickness = (z * 0.1).max(0.3);

            let z_mod = scrolled_z.rem_euclid(grid_size);
            let is_horiz = z_mod < thickness || z_mod > grid_size - thickness;

            for x in 0..self.width {
                let dx = x as f32 - center_x;
                let world_x = dx * z / focal_length;

                let x_mod = world_x.rem_euclid(grid_size);
                let is_vert = x_mod < thickness || x_mod > grid_size - thickness;

                if is_horiz || is_vert {
                    let mut line_intensity = intensity;

                    if is_horiz && is_vert {
                        line_intensity = (line_intensity * 1.5).min(1.0);
                    }

                    let color = interpolate_gradient(&self.palette, line_intensity);

                    let char_idx =
                        ((line_intensity * (chars_len as f32 - 1.0)) as usize).min(chars_len - 1);
                    let c = self.charset.chars[char_idx];

                    buffer.set(x, y as u16, c, color, crossterm::style::Color::Reset);
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
