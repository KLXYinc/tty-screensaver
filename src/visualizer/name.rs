use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemePalette;
use crate::visualizer::Visualizer;
use crossterm::style::Color;
use std::f64::consts::PI;

const NAME_ART: [&str; 5] = [
    "### ### #   # ### #   # ",
    "#   #    # #  # # ##  # ",
    "##  ##    #   # # # # # ",
    "#   #    # #  # # #  ## ",
    "#   ### #   # ### #   # ",
];

#[derive(Clone, Copy)]
enum NameMode {
    Cylinder,
    Wave,
    Bounce,
}

const MODES: [NameMode; 3] = [NameMode::Cylinder, NameMode::Wave, NameMode::Bounce];

pub struct NameVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    time: f64,
    mode_idx: usize,
    bounce_x: f64,
    bounce_y: f64,
    bounce_dx: f64,
    bounce_dy: f64,
}

impl NameVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        Self {
            width,
            height,
            palette,
            charset,
            time: 0.0,
            mode_idx: 0,
            bounce_x: width as f64 / 2.0,
            bounce_y: height as f64 / 2.0,
            bounce_dx: 20.0,
            bounce_dy: 10.0,
        }
    }

    fn get_char(&self, val: f64) -> char {
        if self.charset.chars.is_empty() {
            return '█';
        }
        let idx = (val * self.charset.chars.len() as f64) as usize;
        self.charset.chars[idx.clamp(0, self.charset.chars.len() - 1)]
    }
}

impl Visualizer for NameVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.time += delta_time;

        self.bounce_x += self.bounce_dx * delta_time;
        self.bounce_y += self.bounce_dy * delta_time;

        let art_w = NAME_ART[0].len() as f64;
        let art_h = NAME_ART.len() as f64;

        let scale_x = 2.0;
        let scale_y = 2.0;

        let bound_x = self.width as f64 - art_w * scale_x;
        let bound_y = self.height as f64 - art_h * scale_y;

        if self.bounce_x < 0.0 && self.bounce_dx < 0.0 {
            self.bounce_dx *= -1.0;
        }
        if self.bounce_x > bound_x && self.bounce_dx > 0.0 {
            self.bounce_dx *= -1.0;
        }
        if self.bounce_y < 0.0 && self.bounce_dy < 0.0 {
            self.bounce_dy *= -1.0;
        }
        if self.bounce_y > bound_y && self.bounce_dy > 0.0 {
            self.bounce_dy *= -1.0;
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.bounce_x = self.width as f64 / 2.0;
            self.bounce_y = self.height as f64 / 2.0;
        }

        buffer.clear();

        let mut z_buffer = vec![f64::INFINITY; (self.width as usize) * (self.height as usize)];

        let art_w = NAME_ART[0].len() as f64;
        let art_h = NAME_ART.len() as f64;

        match MODES[self.mode_idx] {
            NameMode::Cylinder => {
                let rot_y = -self.time * 2.0;
                let rot_x = (self.time * 0.8).sin() * 0.4;
                let rot_z = (self.time * 0.5).sin() * 0.2;

                let sin_y = rot_y.sin();
                let cos_y = rot_y.cos();
                let sin_x = rot_x.sin();
                let cos_x = rot_x.cos();
                let sin_z = rot_z.sin();
                let cos_z = rot_z.cos();

                let aspect = self.width as f64 / self.height as f64;
                let cylinder_radius = 15.0;
                let repeat = 3;

                for rep in 0..repeat {
                    for (y, row) in NAME_ART.iter().enumerate() {
                        for (x, ch) in row.chars().enumerate() {
                            if ch == '#' {
                                let total_w = art_w * repeat as f64;
                                let global_x = x as f64 + (rep as f64 * art_w);

                                let theta = (global_x / total_w) * PI * 2.0;

                                let lx = theta.cos() * cylinder_radius;
                                let lz = theta.sin() * cylinder_radius;
                                let ly = (y as f64 - (art_h / 2.0)) * 2.0;

                                let x1 = lx * cos_y - lz * sin_y;
                                let z1 = lx * sin_y + lz * cos_y;
                                let y2 = ly * cos_x - z1 * sin_x;
                                let z2 = ly * sin_x + z1 * cos_x;
                                let x3 = x1 * cos_z - y2 * sin_z;
                                let y3 = x1 * sin_z + y2 * cos_z;

                                let distance = 40.0;
                                let z_factor = distance / (distance + z2);

                                if z_factor < 0.0 {
                                    continue;
                                }

                                let scale = 20.0;
                                let proj_x = (x3 / scale) * z_factor;
                                let proj_y = (y3 / scale) * z_factor;

                                let screen_x = ((proj_x / aspect) * 0.5 + 0.5) * self.width as f64;
                                let screen_y = (proj_y * 0.5 + 0.5) * self.height as f64;

                                let depth = z2;

                                let nz1 = lx * sin_y + lz * cos_y;
                                let nz2 = nz1 * cos_x;

                                if nz2 > 0.0 {
                                    continue;
                                }

                                let dim =
                                    (1.0 - (depth + cylinder_radius) / (cylinder_radius * 2.0))
                                        .clamp(0.2, 1.0) as f32;
                                let hue = (theta / (PI * 2.0)).fract();

                                let color =
                                    crate::themes::interpolate_gradient(&self.palette, hue as f32);
                                let final_color = match color {
                                    Color::Rgb { r, g, b } => Color::Rgb {
                                        r: (r as f32 * dim) as u8,
                                        g: (g as f32 * dim) as u8,
                                        b: (b as f32 * dim) as u8,
                                    },
                                    _ => color,
                                };

                                let draw_ch = self.get_char(hue);

                                let sx = screen_x.round() as i32;
                                let sy = screen_y.round() as i32;

                                for dx in 0..=1 {
                                    let dsx = sx + dx;
                                    if dsx >= 0
                                        && dsx < self.width as i32
                                        && sy >= 0
                                        && sy < self.height as i32
                                    {
                                        let idx = (sy * self.width as i32 + dsx) as usize;
                                        if depth < z_buffer[idx] {
                                            z_buffer[idx] = depth;
                                            buffer.set(
                                                dsx as u16,
                                                sy as u16,
                                                draw_ch,
                                                final_color,
                                                Color::Reset,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            NameMode::Wave => {
                let scale_x = 2.0;
                let scale_y = 2.0;

                let start_x = (self.width as f64 - art_w * scale_x) / 2.0;
                let start_y = (self.height as f64 - art_h * scale_y) / 2.0;

                for (y, row) in NAME_ART.iter().enumerate() {
                    for (x, ch) in row.chars().enumerate() {
                        if ch == '#' {
                            let wave_y = (x as f64 * 0.3 + self.time * 5.0).sin() * 3.0;
                            let screen_x = start_x + x as f64 * scale_x;
                            let screen_y = start_y + y as f64 * scale_y + wave_y;

                            let hue = ((x as f64 / art_w) + self.time).fract();
                            let color =
                                crate::themes::interpolate_gradient(&self.palette, hue as f32);
                            let draw_ch = self.get_char(hue);

                            let sx = screen_x.round() as i32;
                            let sy = screen_y.round() as i32;

                            if sx >= 0
                                && sx < self.width as i32
                                && sy >= 0
                                && sy < self.height as i32
                            {
                                buffer.set(sx as u16, sy as u16, draw_ch, color, Color::Reset);
                            }
                            if sx + 1 >= 0
                                && sx + 1 < self.width as i32
                                && sy >= 0
                                && sy < self.height as i32
                            {
                                buffer.set(
                                    (sx + 1) as u16,
                                    sy as u16,
                                    draw_ch,
                                    color,
                                    Color::Reset,
                                );
                            }
                        }
                    }
                }
            }
            NameMode::Bounce => {
                let scale_x = 2.0;
                let scale_y = 2.0;

                for (y, row) in NAME_ART.iter().enumerate() {
                    for (x, ch) in row.chars().enumerate() {
                        if ch == '#' {
                            let screen_x = self.bounce_x + x as f64 * scale_x;
                            let screen_y = self.bounce_y + y as f64 * scale_y;

                            let hue = ((x as f64 / art_w) + self.time * 2.0).fract();
                            let color =
                                crate::themes::interpolate_gradient(&self.palette, hue as f32);
                            let draw_ch = self.get_char(hue);

                            let sx = screen_x.round() as i32;
                            let sy = screen_y.round() as i32;
                            if sx >= 0
                                && sx < self.width as i32
                                && sy >= 0
                                && sy < self.height as i32
                            {
                                buffer.set(sx as u16, sy as u16, draw_ch, color, Color::Reset);
                            }
                            if sx + 1 >= 0
                                && sx + 1 < self.width as i32
                                && sy >= 0
                                && sy < self.height as i32
                            {
                                buffer.set(
                                    (sx + 1) as u16,
                                    sy as u16,
                                    draw_ch,
                                    color,
                                    Color::Reset,
                                );
                            }
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
        if delta > 0 {
            self.mode_idx = (self.mode_idx + 1) % MODES.len();
        } else if delta < 0 {
            self.mode_idx = (self.mode_idx + MODES.len() - 1) % MODES.len();
        }
    }
}
