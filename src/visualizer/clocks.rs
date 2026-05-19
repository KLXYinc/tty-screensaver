use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemePalette;
use crate::visualizer::Visualizer;
use crossterm::style::Color;
use std::time::{SystemTime, UNIX_EPOCH};

const DIGITS: [&[&str]; 11] = [
    &["###", "# #", "# #", "# #", "###"],
    &["  #", "  #", "  #", "  #", "  #"],
    &["###", "  #", "###", "#  ", "###"],
    &["###", "  #", "###", "  #", "###"],
    &["# #", "# #", "###", "  #", "  #"],
    &["###", "#  ", "###", "  #", "###"],
    &["###", "#  ", "###", "# #", "###"],
    &["###", "  #", "  #", "  #", "  #"],
    &["###", "# #", "###", "# #", "###"],
    &["###", "# #", "###", "  #", "###"],
    &["   ", " # ", "   ", " # ", "   "],
];

#[derive(Clone, Copy)]
pub enum ClockType {
    SpinningDigital,
    Spinning3D,
    WavyDigital,
    Digital,
    Binary,
    Analog,
}

const CLOCK_TYPES: [ClockType; 6] = [
    ClockType::SpinningDigital,
    ClockType::Spinning3D,
    ClockType::WavyDigital,
    ClockType::Digital,
    ClockType::Binary,
    ClockType::Analog,
];

pub struct ClocksVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    time: f64,
    clock_idx: usize,
    zoom: f64,
}

impl ClocksVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        Self {
            width,
            height,
            palette,
            charset,
            time: 0.0,
            clock_idx: 0,
            zoom: 1.0,
        }
    }

    fn draw_line_3d(
        &self,
        buffer: &mut ScreenBuffer,
        p1: (f64, f64, f64),
        p2: (f64, f64, f64),
        color: Color,
        ch: char,
    ) {
        let steps = 50;
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let x = p1.0 + (p2.0 - p1.0) * t;
            let y = p1.1 + (p2.1 - p1.1) * t;
            let z = p1.2 + (p2.2 - p1.2) * t;

            let distance = 5.0;
            let z_factor = (distance / (distance + z)) * self.zoom;

            let proj_x = x * z_factor;
            let proj_y = y * z_factor;

            let aspect = self.width as f64 / self.height as f64;
            let screen_x = ((proj_x / aspect) * 0.5 + 0.5) * self.width as f64;
            let screen_y = (proj_y * 0.5 + 0.5) * self.height as f64;

            if screen_x >= 0.0
                && screen_x < self.width as f64
                && screen_y >= 0.0
                && screen_y < self.height as f64
            {
                buffer.set(screen_x as u16, screen_y as u16, ch, color, Color::Reset);
            }
        }
    }

    fn draw_line_2d(
        &self,
        buffer: &mut ScreenBuffer,
        p1: (f64, f64),
        p2: (f64, f64),
        color: Color,
        ch: char,
    ) {
        let steps = 100;
        let cx = self.width as f64 / 2.0;
        let cy = self.height as f64 / 2.0;

        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let x = p1.0 + (p2.0 - p1.0) * t;
            let y = p1.1 + (p2.1 - p1.1) * t;

            let zx = cx + (x - cx) * self.zoom;
            let zy = cy + (y - cy) * self.zoom;

            if zx >= 0.0 && zx < self.width as f64 && zy >= 0.0 && zy < self.height as f64 {
                buffer.set(zx as u16, zy as u16, ch, color, Color::Reset);
            }
        }
    }
}

impl Visualizer for ClocksVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.time += delta_time;
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
        }

        buffer.clear();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let local_secs = now + 3 * 3600;

        let seconds = (local_secs % 60) as i32;
        let minutes = ((local_secs / 60) % 60) as i32;
        let hours_24 = ((local_secs / 3600) % 24) as i32;
        let hours_12 = if hours_24 % 12 == 0 {
            12
        } else {
            hours_24 % 12
        };

        let primary_color = self.palette.colors.get(0).copied().unwrap_or(Color::White);
        let secondary_color = self.palette.colors.get(1).copied().unwrap_or(Color::White);
        let accent_color = self.palette.colors.get(2).copied().unwrap_or(Color::White);

        match CLOCK_TYPES[self.clock_idx] {
            ClockType::SpinningDigital => {
                let time_str = format!("{:02}:{:02}:{:02}", hours_12, minutes, seconds);

                let rot_y = self.time * 1.5;
                let rot_x = (self.time * 0.5).sin() * 0.5;
                let rot_z = (self.time * 0.2).sin() * 0.2;

                let sin_y = rot_y.sin();
                let cos_y = rot_y.cos();
                let sin_x = rot_x.sin();
                let cos_x = rot_x.cos();
                let sin_z = rot_z.sin();
                let cos_z = rot_z.cos();

                let aspect = self.width as f64 / self.height as f64;

                let char_w = 4.0;
                let total_w = time_str.len() as f64 * char_w;

                let mut z_buffer =
                    vec![f64::INFINITY; (self.width as usize) * (self.height as usize)];

                for (i, c) in time_str.chars().enumerate() {
                    let digit_idx = if c == ':' {
                        10
                    } else {
                        c.to_digit(10).unwrap() as usize
                    };
                    let digit_data = DIGITS[digit_idx];
                    let color = if i < 2 {
                        primary_color
                    } else if i < 5 {
                        secondary_color
                    } else {
                        accent_color
                    };

                    for (dy, row) in digit_data.iter().enumerate() {
                        for (dx, ch) in row.chars().enumerate() {
                            if ch == '#' {
                                let lx = (i as f64 * char_w) + dx as f64 - (total_w / 2.0);
                                let ly = dy as f64 - 2.5;

                                for lz in -1..=1 {
                                    let x = lx * 0.8;
                                    let y = ly * 0.8;
                                    let z = lz as f64 * 0.8;

                                    let x1 = x * cos_y - z * sin_y;
                                    let z1 = x * sin_y + z * cos_y;
                                    let y2 = y * cos_x - z1 * sin_x;
                                    let z2 = y * sin_x + z1 * cos_x;
                                    let x3 = x1 * cos_z - y2 * sin_z;
                                    let y3 = x1 * sin_z + y2 * cos_z;

                                    let distance = 30.0;
                                    let z_factor = (distance / (distance + z2)) * self.zoom;

                                    if z_factor < 0.0 {
                                        continue;
                                    }

                                    let proj_x = x3 * z_factor;
                                    let proj_y = y3 * z_factor;

                                    let screen_x =
                                        ((proj_x / aspect) * 0.5 + 0.5) * self.width as f64;
                                    let screen_y = (proj_y * 0.5 + 0.5) * self.height as f64;

                                    let depth = z2;
                                    let radius = (2.0 * z_factor).clamp(1.0, 4.0) as i32;

                                    let dim = (1.0 - (depth + 10.0) / 20.0).clamp(0.2, 1.0) as f32;
                                    let final_color = match color {
                                        Color::Rgb { r, g, b } => Color::Rgb {
                                            r: (r as f32 * dim) as u8,
                                            g: (g as f32 * dim) as u8,
                                            b: (b as f32 * dim) as u8,
                                        },
                                        _ => color,
                                    };

                                    for ddy in -radius..=radius {
                                        for ddx in -radius..=radius {
                                            if ddx * ddx + ddy * ddy <= radius * radius {
                                                let dx_scr = screen_x as i32 + ddx;
                                                let dy_scr = screen_y as i32 + ddy;
                                                if dx_scr >= 0
                                                    && dx_scr < self.width as i32
                                                    && dy_scr >= 0
                                                    && dy_scr < self.height as i32
                                                {
                                                    let idx = (dy_scr * self.width as i32 + dx_scr)
                                                        as usize;
                                                    if depth < z_buffer[idx] {
                                                        z_buffer[idx] = depth;
                                                        buffer.set(
                                                            dx_scr as u16,
                                                            dy_scr as u16,
                                                            '█',
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
                    }
                }
            }
            ClockType::Spinning3D => {
                let sec_f = seconds as f64;
                let min_f = minutes as f64;
                let hr_f = hours_12 as f64;

                let sec_angle =
                    sec_f * (std::f64::consts::PI * 2.0 / 60.0) - std::f64::consts::PI / 2.0;
                let min_angle = (min_f + sec_f / 60.0) * (std::f64::consts::PI * 2.0 / 60.0)
                    - std::f64::consts::PI / 2.0;
                let hr_angle = (hr_f + min_f / 60.0) * (std::f64::consts::PI * 2.0 / 12.0)
                    - std::f64::consts::PI / 2.0;

                let rot_y = self.time * 0.5;
                let rot_x = (self.time * 0.3).sin() * 0.3;

                let rotate = |x: f64, y: f64, z: f64| -> (f64, f64, f64) {
                    let y1 = y * rot_x.cos() - z * rot_x.sin();
                    let z1 = y * rot_x.sin() + z * rot_x.cos();
                    let x2 = x * rot_y.cos() + z1 * rot_y.sin();
                    let z2 = -x * rot_y.sin() + z1 * rot_y.cos();
                    (x2, y1, z2)
                };

                let radius = 2.0;
                let segments = 60;
                for i in 0..segments {
                    let angle1 = (i as f64 / segments as f64) * std::f64::consts::PI * 2.0;
                    let angle2 = ((i + 1) as f64 / segments as f64) * std::f64::consts::PI * 2.0;
                    let p1 = rotate(angle1.cos() * radius, angle1.sin() * radius, 0.0);
                    let p2 = rotate(angle2.cos() * radius, angle2.sin() * radius, 0.0);
                    let ch = if i % 5 == 0 { 'O' } else { '.' };
                    self.draw_line_3d(buffer, p1, p2, primary_color, ch);
                }

                let center = rotate(0.0, 0.0, 0.0);

                let hx = hr_angle.cos() * radius * 0.5;
                let hy = hr_angle.sin() * radius * 0.5;
                self.draw_line_3d(buffer, center, rotate(hx, hy, 0.1), secondary_color, 'H');

                let mx = min_angle.cos() * radius * 0.8;
                let my = min_angle.sin() * radius * 0.8;
                self.draw_line_3d(buffer, center, rotate(mx, my, 0.2), primary_color, 'M');

                let sx = sec_angle.cos() * radius * 0.9;
                let sy = sec_angle.sin() * radius * 0.9;
                self.draw_line_3d(buffer, center, rotate(sx, sy, 0.3), accent_color, '*');
            }
            ClockType::WavyDigital | ClockType::Digital => {
                let time_str = format!("{:02}:{:02}:{:02}", hours_12, minutes, seconds);
                let char_w = 4;
                let total_w = (time_str.len() * char_w) as f64;

                let cx = self.width as f64 / 2.0;
                let cy = self.height as f64 / 2.0;

                let is_wavy = matches!(CLOCK_TYPES[self.clock_idx], ClockType::WavyDigital);

                for (i, c) in time_str.chars().enumerate() {
                    let digit_idx = if c == ':' {
                        10
                    } else {
                        c.to_digit(10).unwrap() as usize
                    };
                    let digit_data = DIGITS[digit_idx];

                    for (dy, row) in digit_data.iter().enumerate() {
                        for (dx, ch) in row.chars().enumerate() {
                            if ch == '#' {
                                let local_x = (i * char_w + dx) as f64 - (total_w / 2.0);
                                let local_y = dy as f64 - 2.5;

                                let wave_offset = if is_wavy {
                                    ((local_x * 0.2 + self.time * 5.0).sin() * 2.0)
                                } else {
                                    0.0
                                };

                                let x = cx + local_x * self.zoom;
                                let y = cy + (local_y + wave_offset) * self.zoom;

                                if x >= 0.0
                                    && x < self.width as f64
                                    && y >= 0.0
                                    && y < self.height as f64
                                {
                                    let draw_ch = if is_wavy {
                                        self.charset.chars[(dx * 10 + dy * 10) as usize
                                            % self.charset.chars.len()]
                                    } else {
                                        '█'
                                    };
                                    let color = if i < 2 {
                                        primary_color
                                    } else if i < 5 {
                                        secondary_color
                                    } else {
                                        accent_color
                                    };
                                    buffer.set(x as u16, y as u16, draw_ch, color, Color::Reset);
                                }
                            }
                        }
                    }
                }
            }
            ClockType::Binary => {
                let vals = [
                    hours_24 / 10,
                    hours_24 % 10,
                    minutes / 10,
                    minutes % 10,
                    seconds / 10,
                    seconds % 10,
                ];

                let cx = self.width as f64 / 2.0;
                let cy = self.height as f64 / 2.0;

                let cols = 6;
                let rows = 4;
                let total_w = cols as f64 * 5.0;
                let total_h = rows as f64 * 2.0;

                for (col, &val) in vals.iter().enumerate() {
                    let color = if col < 2 {
                        primary_color
                    } else if col < 4 {
                        secondary_color
                    } else {
                        accent_color
                    };
                    for row in 0..4 {
                        let bit = (val >> (3 - row)) & 1;

                        let local_x = (col as f64 * 5.0) - (total_w / 2.0);
                        let local_y = (row as f64 * 2.0) - (total_h / 2.0);

                        let x = cx + local_x * self.zoom;
                        let y = cy + local_y * self.zoom;

                        let (ch, c) = if bit == 1 {
                            ('█', color)
                        } else {
                            (
                                'O',
                                Color::Rgb {
                                    r: 50,
                                    g: 50,
                                    b: 50,
                                },
                            )
                        };
                        if x >= 0.0 && x < self.width as f64 && y >= 0.0 && y < self.height as f64 {
                            buffer.set(x as u16, y as u16, ch, c, Color::Reset);
                        }
                    }
                }
            }
            ClockType::Analog => {
                let sec_f = seconds as f64;
                let min_f = minutes as f64;
                let hr_f = hours_12 as f64;

                let sec_angle =
                    sec_f * (std::f64::consts::PI * 2.0 / 60.0) - std::f64::consts::PI / 2.0;
                let min_angle = (min_f + sec_f / 60.0) * (std::f64::consts::PI * 2.0 / 60.0)
                    - std::f64::consts::PI / 2.0;
                let hr_angle = (hr_f + min_f / 60.0) * (std::f64::consts::PI * 2.0 / 12.0)
                    - std::f64::consts::PI / 2.0;

                let cx = self.width as f64 / 2.0;
                let cy = self.height as f64 / 2.0;

                let radius_y = ((self.height as f64 / 2.0) - 2.0) * self.zoom;
                let radius_x = radius_y * 2.0;

                let segments = 60;
                for i in 0..segments {
                    let angle = (i as f64 / segments as f64) * std::f64::consts::PI * 2.0;
                    let x = cx + angle.cos() * radius_x;
                    let y = cy + angle.sin() * radius_y;
                    let ch = if i % 5 == 0 { '█' } else { '.' };
                    if x >= 0.0 && x < self.width as f64 && y >= 0.0 && y < self.height as f64 {
                        buffer.set(x as u16, y as u16, ch, primary_color, Color::Reset);
                    }
                }

                let hx = cx + hr_angle.cos() * radius_x * 0.5;
                let hy = cy + hr_angle.sin() * radius_y * 0.5;
                self.draw_line_2d(buffer, (cx, cy), (hx, hy), secondary_color, '█');

                let mx = cx + min_angle.cos() * radius_x * 0.8;
                let my = cy + min_angle.sin() * radius_y * 0.8;
                self.draw_line_2d(buffer, (cx, cy), (mx, my), primary_color, '▓');

                let sx = cx + sec_angle.cos() * radius_x * 0.9;
                let sy = cy + sec_angle.sin() * radius_y * 0.9;
                self.draw_line_2d(buffer, (cx, cy), (sx, sy), accent_color, '*');
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
            self.clock_idx = (self.clock_idx + 1) % CLOCK_TYPES.len();
        } else if delta < 0 {
            self.clock_idx = (self.clock_idx + CLOCK_TYPES.len() - 1) % CLOCK_TYPES.len();
        }
    }

    fn on_scroll_ext(&mut self, delta: i32, is_ctrl: bool) {
        if is_ctrl {
            if delta > 0 {
                self.zoom *= 1.1;
            } else if delta < 0 {
                self.zoom *= 0.9;
            }
            self.zoom = self.zoom.clamp(0.1, 10.0);
        } else {
            self.on_scroll(delta);
        }
    }
}
