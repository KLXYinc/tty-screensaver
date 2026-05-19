use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

pub struct SandVisualizer {
    grid: Vec<f32>,
    next_grid: Vec<f32>,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,
    draining: bool,
    emitter_x: f64,
    emitter_target: f64,
    emitter_speed: f64,
    stream_width: f64,
    state_timer: f64,
    is_emitting: bool,
}

impl SandVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            grid: vec![0.0; (width as usize) * (height as usize)],
            next_grid: vec![0.0; (width as usize) * (height as usize)],
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
            draining: false,
            emitter_x: (width / 2) as f64,
            emitter_target: (width / 2) as f64,
            emitter_speed: 20.0,
            stream_width: 1.5,
            state_timer: rng.gen_range(2.0..5.0),
            is_emitting: true,
        }
    }

    fn get_cell(&self, x: i32, y: i32) -> f32 {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return 1.0;
        }
        self.grid[(y as usize) * (self.width as usize) + (x as usize)]
    }

    fn set_next(&mut self, x: i32, y: i32, val: f32) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.next_grid[(y as usize) * (self.width as usize) + (x as usize)] = val;
        }
    }
}

impl Visualizer for SandVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.accumulator += delta_time;
        let tick_rate = (1.0 / 30.0) / self.speed_multiplier;

        while self.accumulator >= tick_rate {
            self.accumulator -= tick_rate;
            let mut rng = rand::thread_rng();

            self.next_grid.fill(0.0);

            self.state_timer -= tick_rate;
            if self.state_timer <= 0.0 {
                if self.is_emitting {
                    self.is_emitting = false;
                    self.emitter_target = rng.gen_range(3.0..(self.width as f64 - 3.0).max(4.0));
                    self.emitter_speed = rng.gen_range(20.0..50.0);
                    self.state_timer = rng.gen_range(0.5..2.0);
                } else {
                    self.is_emitting = true;
                    self.stream_width = rng.gen_range(1.0..3.0);
                    self.state_timer = rng.gen_range(2.0..6.0);
                }
            }

            let diff = self.emitter_target - self.emitter_x;
            if diff.abs() > 0.1 {
                self.emitter_x += diff.signum() * self.emitter_speed * tick_rate;
            }

            if self.is_emitting {
                let ex = self.emitter_x as i32;
                if ex >= 0 && ex < self.width as i32 {
                    let half_w = self.stream_width as i32;
                    for dx in -half_w..=half_w {
                        if rng.gen_bool(0.6) {
                            let spawn_x = ex + dx;
                            if spawn_x >= 0 && spawn_x < self.width as i32 {
                                if self.grid[spawn_x as usize] == 0.0 {
                                    self.grid[spawn_x as usize] = 0.1;
                                }
                            }
                        }
                    }
                }
            }

            for y in (0..self.height as i32).rev() {
                for x in 0..self.width as i32 {
                    let cell = self.get_cell(x, y);
                    if cell > 0.0 {
                        let new_age = (cell + 0.02).min(1.0);

                        if self.get_cell(x, y + 1) == 0.0
                            && self.next_grid[((y + 1) * self.width as i32 + x) as usize] == 0.0
                        {
                            self.set_next(x, y + 1, new_age);
                        } else {
                            let dl_empty = self.get_cell(x - 1, y + 1) == 0.0
                                && self.next_grid[((y + 1) * self.width as i32 + x - 1) as usize]
                                    == 0.0;
                            let dr_empty = self.get_cell(x + 1, y + 1) == 0.0
                                && self.next_grid[((y + 1) * self.width as i32 + x + 1) as usize]
                                    == 0.0;

                            if dl_empty && dr_empty {
                                let dir = if rng.gen_bool(0.5) { -1 } else { 1 };
                                self.set_next(x + dir, y + 1, new_age);
                            } else if dl_empty {
                                self.set_next(x - 1, y + 1, new_age);
                            } else if dr_empty {
                                self.set_next(x + 1, y + 1, new_age);
                            } else {
                                self.set_next(x, y, new_age);
                            }
                        }
                    }
                }
            }

            let mut sand_count = 0;
            let mut top_blocked = 0;
            for x in 0..self.width {
                if self.grid[x as usize] > 0.0 {
                    top_blocked += 1;
                }
            }
            for &cell in &self.grid {
                if cell > 0.0 {
                    sand_count += 1;
                }
            }

            let total_volume = self.width as usize * self.height as usize;

            if !self.draining {
                let vol_threshold = total_volume * 65 / 100;
                let top_threshold = (self.width as usize) * 8 / 10;

                if sand_count > vol_threshold || top_blocked > top_threshold {
                    self.draining = true;
                }
            } else {
                let stop_threshold = total_volume * 25 / 100;
                if sand_count < stop_threshold {
                    self.draining = false;
                }
            }

            if self.draining {
                let mid_x = self.width / 2;
                for x in (mid_x - 1)..=(mid_x + 1) {
                    if x > 0 && x < self.width {
                        self.next_grid[((self.height - 1) * self.width + x) as usize] = 0.0;
                    }
                }
            }

            std::mem::swap(&mut self.grid, &mut self.next_grid);
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.grid = vec![0.0; (self.width as usize) * (self.height as usize)];
            self.next_grid = vec![0.0; (self.width as usize) * (self.height as usize)];
        }

        buffer.clear();

        let chars_len = self.charset.chars.len();
        if chars_len == 0 {
            return;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.get_cell(x as i32, y as i32);
                if cell > 0.0 {
                    let color = interpolate_gradient(&self.palette, cell);
                    let char_idx = ((cell * chars_len as f32) as usize).min(chars_len - 1);
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
