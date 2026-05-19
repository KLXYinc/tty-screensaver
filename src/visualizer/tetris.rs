use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use crossterm::style::Color;
use rand::Rng;

const TETROMINOES: [[[(i32, i32); 4]; 4]; 7] = [
    [
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
    ],
    [
        [(0, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (1, 1), (0, 2), (1, 2)],
    ],
    [
        [(2, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 2)],
        [(0, 0), (1, 0), (1, 1), (1, 2)],
    ],
    [
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
    ],
    [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
        [(1, 1), (2, 1), (0, 2), (1, 2)],
        [(0, 0), (0, 1), (1, 1), (1, 2)],
    ],
    [
        [(1, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (1, 2)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
    ],
    [
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (1, 2), (2, 2)],
        [(1, 0), (0, 1), (1, 1), (0, 2)],
    ],
];

const TETRIS_COLORS: [Color; 7] = [
    Color::Rgb {
        r: 0,
        g: 255,
        b: 255,
    },
    Color::Rgb { r: 0, g: 0, b: 255 },
    Color::Rgb {
        r: 255,
        g: 165,
        b: 0,
    },
    Color::Rgb {
        r: 255,
        g: 255,
        b: 0,
    },
    Color::Rgb { r: 0, g: 255, b: 0 },
    Color::Rgb {
        r: 128,
        g: 0,
        b: 128,
    },
    Color::Rgb { r: 255, g: 0, b: 0 },
];

pub struct TetrisVisualizer {
    well: Vec<u8>,
    well_w: i32,
    well_h: i32,
    current_piece: usize,
    rotation: usize,
    target_rot: usize,
    px: i32,
    py: i32,
    target_x: i32,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,
}

impl TetrisVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let well_w = (width as i32 / 2).max(10);
        let well_h = (height as i32).max(10);

        let mut vis = Self {
            well: vec![0; (well_w * well_h) as usize],
            well_w,
            well_h,
            current_piece: 0,
            rotation: 0,
            target_rot: 0,
            px: 0,
            py: 0,
            target_x: 0,
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
        };
        vis.spawn_piece();
        vis
    }

    fn spawn_piece(&mut self) {
        let mut rng = rand::thread_rng();
        self.current_piece = rng.gen_range(0..7);
        self.rotation = 0;
        self.py = 0;

        let mut best_score = f32::NEG_INFINITY;
        let mut best_x = self.well_w / 2;
        let mut best_rot = 0;

        for rot in 0..4 {
            for x in -2..self.well_w {
                if self.is_valid(self.current_piece, rot, x, 0) {
                    let mut drop_y = 0;
                    while self.is_valid(self.current_piece, rot, x, drop_y + 1) {
                        drop_y += 1;
                    }

                    let mut test_well = self.well.clone();
                    for &(dx, dy) in &TETROMINOES[self.current_piece][rot] {
                        let nx = x + dx;
                        let ny = drop_y + dy;
                        if ny >= 0 && ny < self.well_h && nx >= 0 && nx < self.well_w {
                            test_well[(ny * self.well_w + nx) as usize] = 1;
                        }
                    }

                    let mut aggregate_height = 0;
                    let mut complete_lines = 0;
                    let mut holes = 0;
                    let mut bumpiness = 0;
                    let mut column_heights = vec![0; self.well_w as usize];

                    for cx in 0..self.well_w {
                        let mut block_found = false;
                        let mut ch = 0;
                        for cy in 0..self.well_h {
                            if test_well[(cy * self.well_w + cx) as usize] != 0 {
                                if !block_found {
                                    block_found = true;
                                    ch = self.well_h - cy;
                                    column_heights[cx as usize] = ch;
                                    aggregate_height += ch;
                                }
                            } else if block_found {
                                holes += 1;
                            }
                        }
                    }

                    for i in 0..(self.well_w as usize - 1) {
                        bumpiness += (column_heights[i] - column_heights[i + 1]).abs();
                    }

                    for cy in 0..self.well_h {
                        let mut full = true;
                        for cx in 0..self.well_w {
                            if test_well[(cy * self.well_w + cx) as usize] == 0 {
                                full = false;
                                break;
                            }
                        }
                        if full {
                            complete_lines += 1;
                        }
                    }

                    let score = -0.510066 * (aggregate_height as f32)
                        + 0.760666 * (complete_lines as f32)
                        - 0.356630 * (holes as f32)
                        - 0.184483 * (bumpiness as f32);

                    let noise: f32 = rng.gen_range(0.0..0.001);
                    let final_score = score + noise;

                    if final_score > best_score {
                        best_score = final_score;
                        best_x = x;
                        best_rot = rot;
                    }
                }
            }
        }

        self.px = self.well_w / 2;
        self.py = 0;
        self.rotation = 0;
        self.target_x = best_x;
        self.target_rot = best_rot;

        if !self.is_valid(self.current_piece, self.rotation, self.px, self.py) {
            self.well.fill(0);
        }
    }

    fn is_valid(&self, piece: usize, rot: usize, px: i32, py: i32) -> bool {
        for &(dx, dy) in &TETROMINOES[piece][rot] {
            let nx = px + dx;
            let ny = py + dy;
            if nx < 0 || nx >= self.well_w || ny < 0 || ny >= self.well_h {
                return false;
            }
            if self.well[(ny * self.well_w + nx) as usize] != 0 {
                return false;
            }
        }
        true
    }

    fn lock_piece(&mut self) {
        for &(dx, dy) in &TETROMINOES[self.current_piece][self.rotation] {
            let nx = self.px + dx;
            let ny = self.py + dy;
            if ny >= 0 && ny < self.well_h && nx >= 0 && nx < self.well_w {
                self.well[(ny * self.well_w + nx) as usize] = (self.current_piece + 1) as u8;
            }
        }

        let mut y = self.well_h - 1;
        while y >= 0 {
            let mut full = true;
            for x in 0..self.well_w {
                if self.well[(y * self.well_w + x) as usize] == 0 {
                    full = false;
                    break;
                }
            }

            if full {
                for row in (1..=y).rev() {
                    for x in 0..self.well_w {
                        self.well[(row * self.well_w + x) as usize] =
                            self.well[((row - 1) * self.well_w + x) as usize];
                    }
                }
                for x in 0..self.well_w {
                    self.well[x as usize] = 0;
                }
            } else {
                y -= 1;
            }
        }

        self.spawn_piece();
    }
}

impl Visualizer for TetrisVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.accumulator += delta_time;
        let tick_rate = (1.0 / 25.0) / self.speed_multiplier;

        while self.accumulator >= tick_rate {
            self.accumulator -= tick_rate;

            if self.rotation != self.target_rot {
                let next_rot = (self.rotation + 1) % 4;
                if self.is_valid(self.current_piece, next_rot, self.px, self.py) {
                    self.rotation = next_rot;
                }
            }

            for _ in 0..3 {
                if self.px < self.target_x {
                    if self.is_valid(self.current_piece, self.rotation, self.px + 1, self.py) {
                        self.px += 1;
                    }
                } else if self.px > self.target_x {
                    if self.is_valid(self.current_piece, self.rotation, self.px - 1, self.py) {
                        self.px -= 1;
                    }
                }
            }

            if self.is_valid(self.current_piece, self.rotation, self.px, self.py + 1) {
                self.py += 1;
            } else {
                self.lock_piece();
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.well_w = (self.width as i32 / 2).max(10);
            self.well_h = (self.height as i32).max(10);
            self.well = vec![0; (self.well_w * self.well_h) as usize];
            self.spawn_piece();
        }

        buffer.clear();

        let block_w = 2;
        let block_h = 1;

        let scaled_w = self.well_w * block_w;
        let scaled_h = self.well_h * block_h;
        let offset_x = (self.width as i32 - scaled_w) / 2;
        let offset_y = (self.height as i32 - scaled_h) / 2;

        let block_char = '█';

        for y in 0..self.well_h {
            for x in 0..self.well_w {
                let cell = self.well[(y * self.well_w + x) as usize];
                if cell > 0 {
                    let color = interpolate_gradient(&self.palette, (cell as f32) / 7.0);
                    for dy in 0..block_h {
                        for dx in 0..block_w {
                            let draw_x = x * block_w + dx + offset_x;
                            let draw_y = y * block_h + dy + offset_y;
                            if draw_x >= 0
                                && draw_x < self.width as i32
                                && draw_y >= 0
                                && draw_y < self.height as i32
                            {
                                buffer.set(
                                    draw_x as u16,
                                    draw_y as u16,
                                    block_char,
                                    color,
                                    crossterm::style::Color::Reset,
                                );
                            }
                        }
                    }
                }
            }
        }

        let piece_color =
            interpolate_gradient(&self.palette, ((self.current_piece + 1) as f32) / 7.0);
        for &(dx, dy) in &TETROMINOES[self.current_piece][self.rotation] {
            let x = self.px + dx;
            let y = self.py + dy;

            for ddy in 0..block_h {
                for ddx in 0..block_w {
                    let draw_x = x * block_w + ddx + offset_x;
                    let draw_y = y * block_h + ddy + offset_y;
                    if draw_x >= 0
                        && draw_x < self.width as i32
                        && draw_y >= 0
                        && draw_y < self.height as i32
                    {
                        buffer.set(
                            draw_x as u16,
                            draw_y as u16,
                            block_char,
                            piece_color,
                            crossterm::style::Color::Reset,
                        );
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
        self.speed_multiplier += delta as f64 * 0.2;
        self.speed_multiplier = self.speed_multiplier.clamp(0.01, 10000.0);
    }
}
