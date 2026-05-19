use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

pub struct BreakerVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,

    ball_x: f64,
    ball_y: f64,
    ball_vx: f64,
    ball_vy: f64,

    paddle_x: f64,
    paddle_width: f64,

    bricks: Vec<bool>,
    brick_cols: usize,
    brick_rows: usize,
    brick_width: f64,
    brick_height: f64,

    speed_multiplier: f64,
    reset_timer: f64,
}

impl BreakerVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            width,
            height,
            palette,
            charset,
            ball_x: width as f64 / 2.0,
            ball_y: height as f64 / 2.0,
            ball_vx: 15.0,
            ball_vy: 10.0,
            paddle_x: width as f64 / 2.0,
            paddle_width: 10.0,
            bricks: Vec::new(),
            brick_cols: 0,
            brick_rows: 0,
            brick_width: 6.0,
            brick_height: 2.0,
            speed_multiplier: 1.0,
            reset_timer: 0.0,
        };
        vis.init_level();
        vis
    }

    fn init_level(&mut self) {
        if self.width < 10 || self.height < 10 {
            return;
        }

        let mut rng = rand::thread_rng();

        self.brick_cols = (self.width as f64 / self.brick_width).floor() as usize - 2;
        self.brick_rows = (self.height as f64 * 0.3 / self.brick_height).floor() as usize;

        if self.brick_cols == 0 {
            self.brick_cols = 1;
        }
        if self.brick_rows == 0 {
            self.brick_rows = 1;
        }

        self.bricks = vec![true; self.brick_cols * self.brick_rows];

        self.ball_x = self.width as f64 / 2.0;
        self.ball_y = self.height as f64 - 5.0;

        let dir_x = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        self.ball_vx = dir_x * rng.gen_range(15.0..25.0);
        self.ball_vy = -15.0;

        self.paddle_x = self.ball_x;
    }
}

impl Visualizer for BreakerVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = delta_time * self.speed_multiplier;

        if self.reset_timer > 0.0 {
            self.reset_timer -= delta_time;
            if self.reset_timer <= 0.0 {
                self.init_level();
            }
            return;
        }

        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;

        let max_paddle_speed = 40.0;
        let diff = self.ball_x - self.paddle_x;
        if diff.abs() > 0.5 {
            self.paddle_x += diff.signum() * (max_paddle_speed * dt).min(diff.abs());
        }

        let half_pad = self.paddle_width / 2.0;
        self.paddle_x = self
            .paddle_x
            .clamp(half_pad, self.width as f64 - 1.0 - half_pad);

        if self.ball_x <= 0.0 {
            self.ball_x = 0.0;
            self.ball_vx *= -1.0;
        } else if self.ball_x >= self.width as f64 - 1.0 {
            self.ball_x = self.width as f64 - 1.0;
            self.ball_vx *= -1.0;
        }

        if self.ball_y <= 0.0 {
            self.ball_y = 0.0;
            self.ball_vy *= -1.0;
        }

        let paddle_y = self.height as f64 - 2.0;

        if self.ball_y >= paddle_y - 0.5 && self.ball_y <= paddle_y + 0.5 && self.ball_vy > 0.0 {
            if self.ball_x >= self.paddle_x - half_pad - 1.0
                && self.ball_x <= self.paddle_x + half_pad + 1.0
            {
                self.ball_y = paddle_y - 0.5;
                self.ball_vy *= -1.0;

                let hit_offset = (self.ball_x - self.paddle_x) / half_pad;
                self.ball_vx += hit_offset * 10.0;

                self.ball_vx = self.ball_vx.clamp(-35.0, 35.0);
            }
        }

        if self.ball_y >= self.height as f64 {
            self.reset_timer = 1.0;
            return;
        }

        let margin_x = (self.width as f64 - (self.brick_cols as f64 * self.brick_width)) / 2.0;
        let margin_y = 2.0;

        if self.ball_y >= margin_y
            && self.ball_y <= margin_y + (self.brick_rows as f64 * self.brick_height)
        {
            let col = ((self.ball_x - margin_x) / self.brick_width).floor() as i32;
            let row = ((self.ball_y - margin_y) / self.brick_height).floor() as i32;

            if col >= 0 && col < self.brick_cols as i32 && row >= 0 && row < self.brick_rows as i32
            {
                let idx = (row as usize * self.brick_cols) + col as usize;
                if self.bricks[idx] {
                    self.bricks[idx] = false;
                    self.ball_vy *= -1.0;

                    self.ball_vx *= 1.02;
                    self.ball_vy *= 1.02;

                    if !self.bricks.iter().any(|&b| b) {
                        self.reset_timer = 2.0;
                    }
                }
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.init_level();
        }

        buffer.clear();

        if self.charset.chars.is_empty() {
            return;
        }

        let _chars_len = self.charset.chars.len();
        let ball_char = '●';
        let paddle_char = '█';

        let ball_color = interpolate_gradient(&self.palette, 1.0);
        let paddle_color = interpolate_gradient(&self.palette, 0.5);

        let py = self.height as i32 - 2;
        let px = self.paddle_x.round() as i32;
        let half_pad = (self.paddle_width / 2.0) as i32;

        for dx in -half_pad..=half_pad {
            let sx = px + dx;
            if sx >= 0 && sx < self.width as i32 {
                buffer.set(
                    sx as u16,
                    py as u16,
                    paddle_char,
                    paddle_color,
                    crossterm::style::Color::Reset,
                );
            }
        }

        let margin_x = (self.width as f64 - (self.brick_cols as f64 * self.brick_width)) / 2.0;
        let margin_y = 2.0;

        for row in 0..self.brick_rows {
            let row_color_percent = 1.0 - (row as f32 / self.brick_rows as f32);
            let brick_color = interpolate_gradient(&self.palette, row_color_percent);

            for col in 0..self.brick_cols {
                let idx = row * self.brick_cols + col;
                if self.bricks[idx] {
                    let start_x = margin_x + (col as f64 * self.brick_width);
                    let start_y = margin_y + (row as f64 * self.brick_height);

                    for by in 0..self.brick_height as i32 {
                        for bx in 0..self.brick_width as i32 {
                            if bx == self.brick_width as i32 - 1
                                || by == self.brick_height as i32 - 1
                            {
                                continue;
                            }

                            let sx = start_x.round() as i32 + bx;
                            let sy = start_y.round() as i32 + by;

                            if sx >= 0
                                && sx < self.width as i32
                                && sy >= 0
                                && sy < self.height as i32
                            {
                                buffer.set(
                                    sx as u16,
                                    sy as u16,
                                    '█',
                                    brick_color,
                                    crossterm::style::Color::Reset,
                                );
                            }
                        }
                    }
                }
            }
        }

        let bx = self.ball_x.round() as i32;
        let by = self.ball_y.round() as i32;

        if bx >= 0 && bx < self.width as i32 && by >= 0 && by < self.height as i32 {
            buffer.set(
                bx as u16,
                by as u16,
                ball_char,
                ball_color,
                crossterm::style::Color::Reset,
            );
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
