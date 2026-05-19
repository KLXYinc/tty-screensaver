use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;
use std::collections::VecDeque;

struct Snake {
    body: VecDeque<(i32, i32)>,
    length: usize,
    color_offset: f32,
    dead_timer: usize,
}

pub struct SnakeVisualizer {
    snakes: Vec<Snake>,
    food: Vec<(i32, i32)>,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,
}

impl SnakeVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            snakes: Vec::new(),
            food: Vec::new(),
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
        };
        vis.reset();
        vis
    }

    fn reset(&mut self) {
        self.snakes.clear();
        self.food.clear();
        let mut rng = rand::thread_rng();

        for _ in 0..5 {
            let x = rng.gen_range(0..self.width as i32);
            let y = rng.gen_range(0..self.height as i32);
            let mut body = VecDeque::new();
            body.push_back((x, y));

            self.snakes.push(Snake {
                body,
                length: rng.gen_range(5..15),
                color_offset: rng.gen_range(0.0..1.0),
                dead_timer: 0,
            });
        }

        for _ in 0..10 {
            self.spawn_food();
        }
    }

    fn spawn_food(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..self.width as i32);
        let y = rng.gen_range(0..self.height as i32);
        self.food.push((x, y));
    }

    fn get_occupied(snakes: &[Snake]) -> std::collections::HashSet<(i32, i32)> {
        let mut set = std::collections::HashSet::new();
        for snake in snakes {
            if snake.dead_timer > 0 {
                continue;
            }
            for &pos in &snake.body {
                set.insert(pos);
            }
        }
        set
    }
}

impl Visualizer for SnakeVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.accumulator += delta_time;

        let tick_rate = (1.0 / 20.0) / self.speed_multiplier;

        while self.accumulator >= tick_rate {
            self.accumulator -= tick_rate;
            let mut rng = rand::thread_rng();
            let mut new_food_needed = 0;
            let occupied = Self::get_occupied(&self.snakes);

            for snake in &mut self.snakes {
                if snake.dead_timer > 0 {
                    snake.dead_timer -= 1;
                    if snake.dead_timer == 0 {
                        let x = rng.gen_range(0..self.width as i32);
                        let y = rng.gen_range(0..self.height as i32);
                        snake.body.clear();
                        snake.body.push_back((x, y));
                        snake.length = rng.gen_range(5..15);
                    }
                    continue;
                }

                let head = *snake.body.front().unwrap();

                let mut target_food = None;
                let mut min_dist = i32::MAX;

                for (fx, fy) in &self.food {
                    let dx = (fx - head.0)
                        .abs()
                        .min(self.width as i32 - (fx - head.0).abs());
                    let dy = (fy - head.1)
                        .abs()
                        .min(self.height as i32 - (fy - head.1).abs());
                    let dist = dx + dy;
                    if dist < min_dist {
                        min_dist = dist;
                        target_food = Some((*fx, *fy));
                    }
                }

                let dirs = [(0, -1), (0, 1), (-1, 0), (1, 0)];
                let mut best_dir = None;
                let mut best_dist = i32::MAX;

                let mut valid_dirs = Vec::new();

                for &(dx, dy) in &dirs {
                    let mut nx = head.0 + dx;
                    let mut ny = head.1 + dy;

                    if nx < 0 {
                        nx += self.width as i32;
                    }
                    if nx >= self.width as i32 {
                        nx -= self.width as i32;
                    }
                    if ny < 0 {
                        ny += self.height as i32;
                    }
                    if ny >= self.height as i32 {
                        ny -= self.height as i32;
                    }

                    if !occupied.contains(&(nx, ny)) {
                        valid_dirs.push((nx, ny));

                        if let Some((fx, fy)) = target_food {
                            let tx = (fx - nx).abs().min(self.width as i32 - (fx - nx).abs());
                            let ty = (fy - ny).abs().min(self.height as i32 - (fy - ny).abs());
                            let dist = tx + ty;

                            if dist < best_dist {
                                best_dist = dist;
                                best_dir = Some((nx, ny));
                            }
                        }
                    }
                }

                let next_pos = if let Some(dir) = best_dir {
                    dir
                } else if !valid_dirs.is_empty() {
                    valid_dirs[rng.gen_range(0..valid_dirs.len())]
                } else {
                    snake.dead_timer = 30;
                    continue;
                };

                snake.body.push_front(next_pos);

                let mut ate = false;
                self.food.retain(|&(fx, fy)| {
                    if fx == next_pos.0 && fy == next_pos.1 {
                        ate = true;
                        new_food_needed += 1;
                        false
                    } else {
                        true
                    }
                });

                if ate {
                    snake.length += 3;
                }

                while snake.body.len() > snake.length {
                    snake.body.pop_back();
                }
            }

            for _ in 0..new_food_needed {
                self.spawn_food();
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.reset();
        }

        buffer.clear();

        let chars_len = self.charset.chars.len();
        if chars_len == 0 {
            return;
        }

        for &(fx, fy) in &self.food {
            if fx >= 0 && fx < self.width as i32 && fy >= 0 && fy < self.height as i32 {
                let color = interpolate_gradient(&self.palette, 1.0);
                buffer.set(
                    fx as u16,
                    fy as u16,
                    '★',
                    color,
                    crossterm::style::Color::Reset,
                );
            }
        }

        for snake in &self.snakes {
            let is_dead = snake.dead_timer > 0;
            let total = snake.body.len().max(1);
            for (idx, &(x, y)) in snake.body.iter().enumerate() {
                if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                    let base_ratio = 1.0 - (idx as f32 / total as f32);
                    let mut ratio = base_ratio + snake.color_offset;
                    if ratio > 1.0 {
                        ratio -= 1.0;
                    }

                    let mut color = interpolate_gradient(&self.palette, ratio);
                    if is_dead {
                        color = crossterm::style::Color::Rgb { r: 255, g: 0, b: 0 };
                    }

                    let char_idx = ((base_ratio * chars_len as f32) as usize).min(chars_len - 1);
                    let char_to_draw = self.charset.chars[char_idx];

                    buffer.set(
                        x as u16,
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
