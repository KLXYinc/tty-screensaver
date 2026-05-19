use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Wall,
    Path,
    Visited,
    Solution,
}

enum MazeState {
    Generating,
    WaitingForSolve(f64),
    Solving,
    Finished,
}

pub struct MazeVisualizer {
    grid: Vec<CellState>,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,

    state: MazeState,
    gen_stack: Vec<(i32, i32)>,
    solve_queue: VecDeque<(i32, i32)>,
    came_from: HashMap<(i32, i32), (i32, i32)>,
    start_pos: (i32, i32),
    target_pos: (i32, i32),
    finished_timer: f64,
}

impl MazeVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            grid: vec![CellState::Wall; (width as usize) * (height as usize)],
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
            state: MazeState::Generating,
            gen_stack: Vec::new(),
            solve_queue: VecDeque::new(),
            came_from: HashMap::new(),
            start_pos: (0, 0),
            target_pos: (0, 0),
            finished_timer: 0.0,
        };
        vis.init_maze();
        vis
    }

    fn init_maze(&mut self) {
        self.grid.fill(CellState::Wall);
        self.gen_stack.clear();
        self.solve_queue.clear();
        self.came_from.clear();

        let max_x = (if self.width % 2 == 0 {
            self.width as i32 - 3
        } else {
            self.width as i32 - 2
        })
        .max(1);
        let max_y = (if self.height % 2 == 0 {
            self.height as i32 - 3
        } else {
            self.height as i32 - 2
        })
        .max(1);

        let start_pos = (1, 1);
        let end_pos = (max_x, max_y);

        self.start_pos = start_pos;
        self.target_pos = end_pos;
        self.set_cell(start_pos.0, start_pos.1, CellState::Path);
        self.gen_stack.push(start_pos);
        self.state = MazeState::Generating;
    }

    fn set_cell(&mut self, x: i32, y: i32, state: CellState) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.grid[(y * self.width as i32 + x) as usize] = state;
        }
    }

    fn get_cell(&self, x: i32, y: i32) -> CellState {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.grid[(y * self.width as i32 + x) as usize]
        } else {
            CellState::Wall
        }
    }

    fn get_wall_char(&self, x: i32, y: i32) -> char {
        let up = self.get_cell(x, y - 1) == CellState::Wall;
        let down = self.get_cell(x, y + 1) == CellState::Wall;
        let left = self.get_cell(x - 1, y) == CellState::Wall;
        let right = self.get_cell(x + 1, y) == CellState::Wall;

        match (up, down, left, right) {
            (true, true, true, true) => '┼',
            (true, true, true, false) => '┤',
            (true, true, false, true) => '├',
            (true, false, true, true) => '┴',
            (false, true, true, true) => '┬',
            (true, true, false, false) => '│',
            (false, false, true, true) => '─',
            (true, false, true, false) => '┘',
            (true, false, false, true) => '└',
            (false, true, true, false) => '┐',
            (false, true, false, true) => '┌',
            (true, false, false, false) => '╵',
            (false, true, false, false) => '╷',
            (false, false, true, false) => '╴',
            (false, false, false, true) => '╶',
            (false, false, false, false) => '·',
        }
    }
}

impl Visualizer for MazeVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.accumulator += delta_time;
        let tick_rate = (1.0 / 60.0) / self.speed_multiplier;

        while self.accumulator >= tick_rate {
            self.accumulator -= tick_rate;

            match self.state {
                MazeState::Generating => {
                    let mut rng = rand::thread_rng();
                    let max_x = (if self.width % 2 == 0 {
                        self.width as i32 - 3
                    } else {
                        self.width as i32 - 2
                    })
                    .max(1);
                    let max_y = (if self.height % 2 == 0 {
                        self.height as i32 - 3
                    } else {
                        self.height as i32 - 2
                    })
                    .max(1);

                    for _ in 0..250 {
                        if let Some(&(cx, cy)) = self.gen_stack.last() {
                            let mut neighbors = Vec::new();
                            let dirs = [(0, -2), (2, 0), (0, 2), (-2, 0)];
                            for &(dx, dy) in &dirs {
                                let nx = cx + dx;
                                let ny = cy + dy;
                                if nx > 0 && nx <= max_x && ny > 0 && ny <= max_y {
                                    if self.get_cell(nx, ny) == CellState::Wall {
                                        neighbors.push((nx, ny, dx, dy));
                                    }
                                }
                            }

                            if !neighbors.is_empty() {
                                let &(nx, ny, dx, dy) = neighbors.choose(&mut rng).unwrap();
                                self.set_cell(cx + dx / 2, cy + dy / 2, CellState::Path);
                                self.set_cell(nx, ny, CellState::Path);
                                self.gen_stack.push((nx, ny));
                            } else {
                                self.gen_stack.pop();
                            }
                        } else {
                            self.state = MazeState::WaitingForSolve(1.0);
                            break;
                        }
                    }
                }
                MazeState::WaitingForSolve(mut timer) => {
                    timer -= tick_rate;
                    if timer <= 0.0 {
                        self.state = MazeState::Solving;
                        self.solve_queue.push_back(self.start_pos);
                        self.set_cell(self.start_pos.0, self.start_pos.1, CellState::Visited);
                    } else {
                        self.state = MazeState::WaitingForSolve(timer);
                    }
                }
                MazeState::Solving => {
                    for _ in 0..40 {
                        if let Some((cx, cy)) = self.solve_queue.pop_front() {
                            if (cx, cy) == self.target_pos {
                                let mut curr = (cx, cy);
                                while let Some(&prev) = self.came_from.get(&curr) {
                                    self.set_cell(curr.0, curr.1, CellState::Solution);
                                    curr = prev;
                                }
                                self.set_cell(curr.0, curr.1, CellState::Solution);
                                self.state = MazeState::Finished;
                                self.finished_timer = 5.0;
                                break;
                            }

                            let dirs = [(0, -1), (1, 0), (0, 1), (-1, 0)];
                            for &(dx, dy) in &dirs {
                                let nx = cx + dx;
                                let ny = cy + dy;
                                let cell = self.get_cell(nx, ny);
                                if cell == CellState::Path {
                                    self.set_cell(nx, ny, CellState::Visited);
                                    self.came_from.insert((nx, ny), (cx, cy));
                                    self.solve_queue.push_back((nx, ny));
                                }
                            }
                        } else {
                            self.state = MazeState::Finished;
                            self.finished_timer = 2.0;
                            break;
                        }
                    }
                }
                MazeState::Finished => {
                    self.finished_timer -= tick_rate;
                    if self.finished_timer <= 0.0 {
                        self.init_maze();
                    }
                }
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.grid = vec![CellState::Wall; (self.width as usize) * (self.height as usize)];
            self.init_maze();
        }
        buffer.clear();

        let chars_len = self.charset.chars.len();
        if chars_len == 0 {
            return;
        }

        let path_char = self.charset.chars[0];
        let visit_char = self.charset.chars[chars_len / 2];
        let sol_char = self.charset.chars[chars_len - 1];

        let wall_col = interpolate_gradient(&self.palette, 0.4);
        let path_col = interpolate_gradient(&self.palette, 0.15);
        let solver_col = interpolate_gradient(&self.palette, 1.0);

        for y in 0..self.height {
            for x in 0..self.width {
                match self.get_cell(x as i32, y as i32) {
                    CellState::Wall => {
                        let c = self.get_wall_char(x as i32, y as i32);
                        buffer.set(x, y, c, wall_col, crossterm::style::Color::Reset);
                    }
                    CellState::Path => {
                        buffer.set(x, y, path_char, path_col, crossterm::style::Color::Reset)
                    }
                    CellState::Visited => {
                        buffer.set(x, y, visit_char, solver_col, crossterm::style::Color::Reset)
                    }
                    CellState::Solution => {
                        buffer.set(x, y, sol_char, solver_col, crossterm::style::Color::Reset)
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
