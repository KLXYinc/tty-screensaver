use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

const DVD_LOGO: [&str; 3] = [" ▄▄▄  ▄   ▄ ▄▄▄ ", " █  █ █   █ █  █ ", " █▄▄▀ ▀▄▄▄▀ █▄▄▀ "];

pub struct DvdVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    color_offset: f32,
    speed_multiplier: f64,
}

impl DvdVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            width,
            height,
            palette,
            charset,
            x: width as f32 / 2.0,
            y: height as f32 / 2.0,
            vx: 15.0,
            vy: 7.5,
            color_offset: rng.gen_range(0.0..1.0),
            speed_multiplier: 1.0,
        }
    }
}

impl Visualizer for DvdVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = (delta_time * self.speed_multiplier) as f32;

        let logo_h = DVD_LOGO.len() as f32;
        let logo_w = DVD_LOGO[0].chars().count() as f32;

        self.x += self.vx * dt;
        self.y += self.vy * dt;

        let mut bounced = false;

        if self.x < 0.0 {
            self.x = 0.0;
            self.vx *= -1.0;
            bounced = true;
        } else if self.x + logo_w >= self.width as f32 {
            self.x = self.width as f32 - logo_w;
            self.vx *= -1.0;
            bounced = true;
        }

        if self.y < 0.0 {
            self.y = 0.0;
            self.vy *= -1.0;
            bounced = true;
        } else if self.y + logo_h >= self.height as f32 {
            self.y = self.height as f32 - logo_h;
            self.vy *= -1.0;
            bounced = true;
        }

        if bounced {
            self.color_offset = (self.color_offset + 0.38) % 1.0;
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            let logo_h = DVD_LOGO.len() as f32;
            let logo_w = DVD_LOGO[0].chars().count() as f32;
            self.x = self.x.clamp(0.0, (self.width as f32 - logo_w).max(0.0));
            self.y = self.y.clamp(0.0, (self.height as f32 - logo_h).max(0.0));
        }

        buffer.clear();

        let base_x = self.x.round() as i32;
        let base_y = self.y.round() as i32;

        let color = interpolate_gradient(&self.palette, self.color_offset);

        for (ly, line) in DVD_LOGO.iter().enumerate() {
            let mut lx = 0;
            for c in line.chars() {
                if c != ' ' {
                    let screen_x = base_x + lx;
                    let screen_y = base_y + ly as i32;

                    if screen_x >= 0
                        && screen_x < self.width as i32
                        && screen_y >= 0
                        && screen_y < self.height as i32
                    {
                        buffer.set(
                            screen_x as u16,
                            screen_y as u16,
                            c,
                            color,
                            crossterm::style::Color::Reset,
                        );
                    }
                }
                lx += 1;
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
