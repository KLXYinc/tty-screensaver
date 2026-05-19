use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

pub struct LifeVisualizer {
    grid: Vec<f64>,
    next_grid: Vec<f64>,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,
}

impl LifeVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            grid: vec![0.0; (width as usize) * (height as usize)],
            next_grid: vec![0.0; (width as usize) * (height as usize)],
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
        };
        vis.randomize();
        vis
    }

    fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        self.grid.fill(0.0);

        let num_gliders = rng.gen_range(5..15);
        for _ in 0..num_gliders {
            let x = rng.gen_range(0..self.width as i32);
            let y = rng.gen_range(0..self.height as i32);
            self.spawn_pattern(x, y, &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
        }

        let num_soups = rng.gen_range(3..8);
        for _ in 0..num_soups {
            let sx = rng.gen_range(0..self.width as i32);
            let sy = rng.gen_range(0..self.height as i32);
            for dy in 0..15 {
                for dx in 0..15 {
                    if rng.gen_bool(0.4) {
                        let x = (sx + dx).rem_euclid(self.width as i32) as usize;
                        let y = (sy + dy).rem_euclid(self.height as i32) as usize;
                        self.grid[y * (self.width as usize) + x] = 1.0;
                    }
                }
            }
        }
    }

    fn spawn_pattern(&mut self, start_x: i32, start_y: i32, offsets: &[(i32, i32)]) {
        for &(dx, dy) in offsets {
            let x = (start_x + dx).rem_euclid(self.width as i32) as usize;
            let y = (start_y + dy).rem_euclid(self.height as i32) as usize;
            self.grid[y * (self.width as usize) + x] = 1.0;
        }
    }

    fn get_cell(&self, x: i32, y: i32) -> f64 {
        let wrapped_x = x.rem_euclid(self.width as i32) as usize;
        let wrapped_y = y.rem_euclid(self.height as i32) as usize;
        self.grid[wrapped_y * (self.width as usize) + wrapped_x]
    }

    fn count_neighbors(&self, x: i32, y: i32) -> u8 {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if self.get_cell(x + dx, y + dy) > 0.0 {
                    count += 1;
                }
            }
        }
        count
    }
}

impl Visualizer for LifeVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.accumulator += delta_time;
        let tick_rate = (1.0 / 15.0) / self.speed_multiplier;

        while self.accumulator >= tick_rate {
            self.accumulator -= tick_rate;

            let mut all_dead = true;
            let mut rng = rand::thread_rng();

            if rng.gen_bool(0.15) {
                let num_anomalies =
                    ((self.width as f64 * self.height as f64) / 1000.0).max(1.0) as i32;
                for _ in 0..num_anomalies {
                    let sx = rng.gen_range(0..self.width as i32);
                    let sy = rng.gen_range(0..self.height as i32);

                    if rng.gen_bool(0.5) {
                        for dy in 0..3 {
                            for dx in 0..3 {
                                if rng.gen_bool(0.5) {
                                    let x = (sx + dx).rem_euclid(self.width as i32) as usize;
                                    let y = (sy + dy).rem_euclid(self.height as i32) as usize;
                                    self.grid[y * (self.width as usize) + x] = 1.0;
                                }
                            }
                        }
                    } else {
                        let offsets = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
                        for &(dx, dy) in &offsets {
                            let x = (sx + dx).rem_euclid(self.width as i32) as usize;
                            let y = (sy + dy).rem_euclid(self.height as i32) as usize;
                            self.grid[y * (self.width as usize) + x] = 1.0;
                        }
                    }
                }
            }

            for y in 0..self.height {
                for x in 0..self.width {
                    let alive = self.get_cell(x as i32, y as i32) > 0.0;
                    let age = self.get_cell(x as i32, y as i32);
                    let neighbors = self.count_neighbors(x as i32, y as i32);

                    let idx = (y as usize) * (self.width as usize) + (x as usize);

                    if alive {
                        if neighbors < 2 || neighbors > 3 {
                            self.next_grid[idx] = 0.0;
                        } else {
                            let new_age = (age + 0.1).min(1.0);
                            self.next_grid[idx] = new_age;
                            all_dead = false;
                        }
                    } else {
                        if neighbors == 3 {
                            self.next_grid[idx] = 0.1;
                            all_dead = false;
                        } else {
                            self.next_grid[idx] = 0.0;
                        }
                    }
                }
            }

            std::mem::swap(&mut self.grid, &mut self.next_grid);

            if all_dead {
                self.randomize();
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.grid = vec![0.0; (self.width as usize) * (self.height as usize)];
            self.next_grid = vec![0.0; (self.width as usize) * (self.height as usize)];
            self.randomize();
        }

        buffer.clear();

        let chars_len = self.charset.chars.len();
        if chars_len == 0 {
            return;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let age = self.get_cell(x as i32, y as i32);
                if age > 0.0 {
                    let color = interpolate_gradient(&self.palette, age as f32);
                    let char_idx = ((age as f32 * chars_len as f32) as usize).min(chars_len - 1);
                    let char_to_draw = self.charset.chars[char_idx];

                    buffer.set(x, y, char_to_draw, color, crossterm::style::Color::Reset);
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
