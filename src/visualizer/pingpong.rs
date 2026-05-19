use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

pub struct PingPongVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,

    ball_x: f64,
    ball_y: f64,
    ball_vx: f64,
    ball_vy: f64,

    paddle_left_y: f64,
    paddle_right_y: f64,
    paddle_height: f64,

    score_left: u32,
    score_right: u32,

    speed_multiplier: f64,
}

impl PingPongVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            width,
            height,
            palette,
            charset,
            ball_x: width as f64 / 2.0,
            ball_y: height as f64 / 2.0,
            ball_vx: 20.0,
            ball_vy: 10.0,
            paddle_left_y: height as f64 / 2.0,
            paddle_right_y: height as f64 / 2.0,
            paddle_height: 5.0,
            score_left: 0,
            score_right: 0,
            speed_multiplier: 1.0,
        };
        vis.reset_ball();
        vis
    }

    fn reset_ball(&mut self) {
        let mut rng = rand::thread_rng();
        self.ball_x = self.width as f64 / 2.0;
        self.ball_y = self.height as f64 / 2.0;

        let dir_x = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        let dir_y = rng.gen_range(-1.0..1.0);

        self.ball_vx = dir_x * 25.0;
        self.ball_vy = dir_y * 15.0;
    }
}

impl Visualizer for PingPongVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = delta_time * self.speed_multiplier;

        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;

        let max_paddle_speed = 30.0;

        if self.ball_vx < 0.0 {
            let diff = self.ball_y - self.paddle_left_y;
            if diff.abs() > 0.5 {
                self.paddle_left_y += diff.signum() * (max_paddle_speed * dt).min(diff.abs());
            }
        }

        if self.ball_vx > 0.0 {
            let diff = self.ball_y - self.paddle_right_y;
            if diff.abs() > 0.5 {
                self.paddle_right_y += diff.signum() * (max_paddle_speed * dt).min(diff.abs());
            }
        }

        let half_pad = self.paddle_height / 2.0;
        self.paddle_left_y = self
            .paddle_left_y
            .clamp(half_pad, self.height as f64 - 1.0 - half_pad);
        self.paddle_right_y = self
            .paddle_right_y
            .clamp(half_pad, self.height as f64 - 1.0 - half_pad);

        if self.ball_y <= 0.0 {
            self.ball_y = 0.0;
            self.ball_vy *= -1.0;
        } else if self.ball_y >= self.height as f64 - 1.0 {
            self.ball_y = self.height as f64 - 1.0;
            self.ball_vy *= -1.0;
        }

        let left_paddle_x = 2.0;
        let right_paddle_x = self.width as f64 - 3.0;

        if self.ball_x <= left_paddle_x + 1.0 && self.ball_x >= left_paddle_x - 1.0 {
            if (self.ball_y - self.paddle_left_y).abs() <= half_pad + 0.5 {
                self.ball_x = left_paddle_x + 1.0;
                self.ball_vx *= -1.05;
                self.ball_vy += (self.ball_y - self.paddle_left_y) * 2.0;
            }
        }

        if self.ball_x >= right_paddle_x - 1.0 && self.ball_x <= right_paddle_x + 1.0 {
            if (self.ball_y - self.paddle_right_y).abs() <= half_pad + 0.5 {
                self.ball_x = right_paddle_x - 1.0;
                self.ball_vx *= -1.05;
                self.ball_vy += (self.ball_y - self.paddle_right_y) * 2.0;
            }
        }

        if self.ball_x < 0.0 {
            self.score_right += 1;
            self.reset_ball();
        } else if self.ball_x > self.width as f64 {
            self.score_left += 1;
            self.reset_ball();
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.reset_ball();
        }

        buffer.clear();

        if self.charset.chars.is_empty() {
            return;
        }

        let _chars_len = self.charset.chars.len();
        let ball_char = '●';
        let paddle_char = '█';
        let net_char = '┆';

        let ball_color = interpolate_gradient(&self.palette, 1.0);
        let paddle_color = interpolate_gradient(&self.palette, 0.7);
        let net_color = interpolate_gradient(&self.palette, 0.3);

        let center_x = self.width / 2;
        for y in 0..self.height {
            if y % 2 == 0 {
                buffer.set(
                    center_x,
                    y,
                    net_char,
                    net_color,
                    crossterm::style::Color::Reset,
                );
            }
        }

        let score_str = format!(" {} : {} ", self.score_left, self.score_right);
        let score_x = center_x as i32 - (score_str.len() as i32 / 2);
        for (i, c) in score_str.chars().enumerate() {
            let sx = score_x + i as i32;
            if sx >= 0 && sx < self.width as i32 {
                buffer.set(
                    sx as u16,
                    1,
                    c,
                    interpolate_gradient(&self.palette, 0.8),
                    crossterm::style::Color::Reset,
                );
            }
        }

        let left_px = 2;
        let right_px = self.width - 3;

        let half_pad = (self.paddle_height / 2.0) as i32;

        let ly = self.paddle_left_y.round() as i32;
        for dy in -half_pad..=half_pad {
            let py = ly + dy;
            if py >= 0 && py < self.height as i32 {
                buffer.set(
                    left_px,
                    py as u16,
                    paddle_char,
                    paddle_color,
                    crossterm::style::Color::Reset,
                );
            }
        }

        let ry = self.paddle_right_y.round() as i32;
        for dy in -half_pad..=half_pad {
            let py = ry + dy;
            if py >= 0 && py < self.height as i32 {
                buffer.set(
                    right_px,
                    py as u16,
                    paddle_char,
                    paddle_color,
                    crossterm::style::Color::Reset,
                );
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
