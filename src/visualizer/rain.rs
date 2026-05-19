use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

struct Drop {
    x: f64,
    y: f64,
    speed: f64,
    z: f64,
}

struct Splash {
    x: u16,
    y: u16,
    life: f64,
}

pub struct RainVisualizer {
    drops: Vec<Drop>,
    splashes: Vec<Splash>,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    speed_multiplier: f64,
}

impl RainVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut visualizer = Self {
            drops: Vec::new(),
            splashes: Vec::new(),
            width,
            height,
            palette,
            charset,
            speed_multiplier: 1.0,
        };
        visualizer.init_drops();
        visualizer
    }

    fn init_drops(&mut self) {
        self.drops.clear();
        let mut rng = rand::thread_rng();
        let num_drops = (self.width as usize * self.height as usize) / 20;

        for _ in 0..num_drops {
            self.drops.push(Drop {
                x: rng.gen_range(0.0..self.width as f64 * 1.5),
                y: rng.gen_range(0.0..self.height as f64),
                speed: rng.gen_range(40.0..80.0),
                z: rng.gen_range(0.3..1.0),
            });
        }
    }
}

impl Visualizer for RainVisualizer {
    fn update(&mut self, mut delta_time: f64) {
        delta_time *= self.speed_multiplier;

        let mut rng = rand::thread_rng();
        let slant = 0.5;

        for drop in &mut self.drops {
            drop.y += drop.speed * drop.z * delta_time;
            drop.x -= drop.speed * drop.z * slant * delta_time;

            if drop.y >= self.height as f64 || drop.x < 0.0 {
                if drop.y >= self.height as f64
                    && rng.gen_bool(0.3)
                    && drop.x >= 0.0
                    && drop.x < self.width as f64
                {
                    self.splashes.push(Splash {
                        x: drop.x as u16,
                        y: self.height - 1,
                        life: 0.3,
                    });
                }

                drop.y = rng.gen_range(-5.0..0.0);
                drop.x = rng.gen_range(0.0..self.width as f64 * 1.5);
                drop.z = rng.gen_range(0.3..1.0);
            }
        }

        for splash in &mut self.splashes {
            splash.life -= delta_time;
        }
        self.splashes.retain(|s| s.life > 0.0);
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.init_drops();
        }

        buffer.clear();

        if self.charset.chars.is_empty() {
            return;
        }

        let chars_len = self.charset.chars.len();

        for drop in &self.drops {
            let x = drop.x as u16;
            let y = drop.y as u16;

            if x < self.width && y < self.height {
                let ratio = drop.z as f32;
                let color = interpolate_gradient(&self.palette, ratio);

                let char_to_draw =
                    if self.charset.name == "Classic" || self.charset.name == "Symbols" {
                        if drop.z > 0.7 { '/' } else { ',' }
                    } else {
                        let char_idx = ((ratio * chars_len as f32) as usize).min(chars_len - 1);
                        self.charset.chars[char_idx]
                    };

                buffer.set(x, y, char_to_draw, color, crossterm::style::Color::Reset);
            }
        }

        for splash in &self.splashes {
            if splash.x < self.width && splash.y < self.height {
                let ratio = (splash.life / 0.3) as f32;
                let color = interpolate_gradient(&self.palette, ratio);
                buffer.set(
                    splash.x,
                    splash.y,
                    'v',
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
        self.speed_multiplier = self.speed_multiplier.clamp(0.01, 10000.0);
    }
}
