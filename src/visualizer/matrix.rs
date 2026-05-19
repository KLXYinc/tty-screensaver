use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

struct Drop {
    x: u16,
    y: f64,
    speed: f64,
    length: u16,
    chars: Vec<char>,
}

pub struct MatrixVisualizer {
    drops: Vec<Drop>,
    palette: ThemePalette,
    charset: CharSet,
    width: u16,
    height: u16,
    speed_multiplier: f64,
}

impl MatrixVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut visualizer = Self {
            drops: Vec::new(),
            palette,
            charset,
            width,
            height,
            speed_multiplier: 1.0,
        };
        visualizer.init_drops();
        visualizer
    }

    fn init_drops(&mut self) {
        self.drops.clear();
        let mut rng = rand::thread_rng();
        for x in 0..self.width {
            if rng.gen_bool(0.7) {
                self.drops
                    .push(self.create_drop(x, rng.gen_range(-20.0..self.height as f64)));
            }
        }
    }

    fn create_drop(&self, x: u16, start_y: f64) -> Drop {
        let mut rng = rand::thread_rng();
        let length = rng.gen_range(10..30);
        let mut chars = Vec::with_capacity(length as usize);
        for _ in 0..length {
            chars.push(self.random_char());
        }
        Drop {
            x,
            y: start_y,
            speed: rng.gen_range(10.0..30.0),
            length,
            chars,
        }
    }

    fn random_char(&self) -> char {
        let mut rng = rand::thread_rng();
        if self.charset.chars.is_empty() {
            return ' ';
        }
        self.charset.chars[rng.gen_range(0..self.charset.chars.len())]
    }
}

impl Visualizer for MatrixVisualizer {
    fn update(&mut self, mut delta_time: f64) {
        delta_time *= self.speed_multiplier;

        let mut rng = rand::thread_rng();

        for drop in &mut self.drops {
            drop.y += drop.speed * delta_time;

            if rng.gen_bool(0.1) {
                let idx = rng.gen_range(0..drop.length) as usize;

                let new_char = if self.charset.chars.is_empty() {
                    ' '
                } else {
                    self.charset.chars[rng.gen_range(0..self.charset.chars.len())]
                };

                drop.chars[idx] = new_char;
            }
        }

        self.drops
            .retain(|d| d.y - (d.length as f64) < self.height as f64);

        while self.drops.len() < (self.width as usize) * 3 / 4 {
            let x = rng.gen_range(0..self.width);
            self.drops
                .push(self.create_drop(x, rng.gen_range(-20.0..0.0)));
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.init_drops();
        }

        buffer.clear();

        for drop in &self.drops {
            let head_y = drop.y as i32;

            for i in 0..drop.length {
                let y = head_y - i as i32;
                if y >= 0 && y < self.height as i32 {
                    let ratio = 1.0 - (i as f32 / drop.length as f32);

                    let color = if i == 0 {
                        crossterm::style::Color::White
                    } else {
                        interpolate_gradient(&self.palette, ratio)
                    };

                    let char_to_draw = drop.chars[i as usize];

                    buffer.set(
                        drop.x,
                        y as u16,
                        char_to_draw,
                        color,
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
