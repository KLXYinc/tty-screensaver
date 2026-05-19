use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemePalette;
use crate::visualizer::Visualizer;
use rand::Rng;

const MAZE: [&str; 31] = [
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#o####.#####.##.#####.####o#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.##### ## #####.######",
    "     #.##### ## #####.#     ",
    "     #.##          ##.#     ",
    "     #.## ###--### ##.#     ",
    "######.## #      # ##.######",
    "      .   #      #   .      ",
    "######.## #      # ##.######",
    "     #.## ######## ##.#     ",
    "     #.##          ##.#     ",
    "     #.## ######## ##.#     ",
    "######.## ######## ##.######",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#.####.#####.##.#####.####.#",
    "#o..##.......  .......##..o#",
    "###.##.##.########.##.##.###",
    "###.##.##.########.##.##.###",
    "#......##....##....##......#",
    "#.##########.##.##########.#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
];

const MAZE_W: i32 = 28;
const MAZE_H: i32 = 31;

struct Entity {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    is_pacman: bool,
}

pub struct PacmanVisualizer {
    dots: Vec<u8>,
    entities: Vec<Entity>,
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    speed_multiplier: f64,
    accumulator: f64,
}

impl PacmanVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            dots: vec![0; (MAZE_W * MAZE_H) as usize],
            entities: Vec::new(),
            width,
            height,
            palette,
            charset,
            speed_multiplier: 1.0,
            accumulator: 0.0,
        };
        vis.reset();
        vis
    }

    fn reset(&mut self) {
        for y in 0..MAZE_H {
            let row = MAZE[y as usize].as_bytes();
            for x in 0..MAZE_W {
                let cell = row[x as usize];
                let val = match cell {
                    b'#' | b'-' => 3,
                    b'.' => 1,
                    b'o' => 2,
                    _ => 0,
                };
                self.dots[(y * MAZE_W + x) as usize] = val;
            }
        }

        self.entities.clear();

        self.entities.push(Entity {
            x: 14.0,
            y: 23.0,
            vx: 8.0,
            vy: 0.0,
            is_pacman: true,
        });

        for _ in 0..4 {
            let mut rng = rand::thread_rng();
            self.entities.push(Entity {
                x: rng.gen_range(13.0..15.0),
                y: rng.gen_range(13.0..15.0),
                vx: 0.0,
                vy: -5.0,
                is_pacman: false,
            });
        }
    }

    fn is_wall(dots: &[u8], x: i32, y: i32) -> bool {
        if x < 0 || x >= MAZE_W {
            return false;
        }
        if y < 0 || y >= MAZE_H {
            return true;
        }
        dots[(y * MAZE_W + x) as usize] == 3
    }
}

impl Visualizer for PacmanVisualizer {
    fn update(&mut self, mut delta_time: f64) {
        delta_time *= self.speed_multiplier;
        self.accumulator += delta_time;
        let mut rng = rand::thread_rng();

        let mut all_dots_eaten = true;
        for &dot in &self.dots {
            if dot == 1 || dot == 2 {
                all_dots_eaten = false;
                break;
            }
        }

        if all_dots_eaten {
            self.reset();
            return;
        }

        for entity in &mut self.entities {
            let next_x = entity.x + entity.vx * delta_time;
            let next_y = entity.y + entity.vy * delta_time;

            let grid_x = next_x.round() as i32;
            let grid_y = next_y.round() as i32;

            if grid_x < 0 {
                entity.x += MAZE_W as f64;
            } else if grid_x >= MAZE_W {
                entity.x -= MAZE_W as f64;
            } else {
                if !Self::is_wall(&self.dots, grid_x, grid_y) {
                    entity.x = next_x;
                    entity.y = next_y;
                } else {
                    let mut possible_dirs = Vec::new();
                    let current_gx = entity.x.round() as i32;
                    let current_gy = entity.y.round() as i32;

                    if !Self::is_wall(&self.dots, current_gx + 1, current_gy) {
                        possible_dirs.push((1.0, 0.0));
                    }
                    if !Self::is_wall(&self.dots, current_gx - 1, current_gy) {
                        possible_dirs.push((-1.0, 0.0));
                    }
                    if !Self::is_wall(&self.dots, current_gx, current_gy + 1) {
                        possible_dirs.push((0.0, 1.0));
                    }
                    if !Self::is_wall(&self.dots, current_gx, current_gy - 1) {
                        possible_dirs.push((0.0, -1.0));
                    }

                    if !possible_dirs.is_empty() {
                        let idx = rng.gen_range(0..possible_dirs.len());
                        let speed = if entity.is_pacman { 8.0 } else { 7.0 };
                        entity.vx = possible_dirs[idx].0 * speed;
                        entity.vy = possible_dirs[idx].1 * speed;
                    }
                }
            }

            if entity.is_pacman {
                let px = entity.x.round() as i32;
                let py = entity.y.round() as i32;
                if px >= 0 && px < MAZE_W && py >= 0 && py < MAZE_H {
                    let cell = &mut self.dots[(py * MAZE_W + px) as usize];
                    if *cell == 1 || *cell == 2 {
                        *cell = 0;
                    }
                }
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
        }

        buffer.clear();

        let wall_char = '█';
        let dot_char = '·';
        let ghost_char = 'M';
        let pacman_char = if (self.accumulator * 15.0) as i32 % 2 == 0 {
            'C'
        } else {
            'O'
        };

        let flash_big_dot = (self.accumulator * 8.0).sin() > 0.0;
        let big_dot_char = if flash_big_dot { '●' } else { ' ' };

        let scale_x = self.width as f64 / MAZE_W as f64;
        let scale_y = self.height as f64 / MAZE_H as f64;

        for y in 0..MAZE_H {
            for x in 0..MAZE_W {
                let cell = self.dots[(y * MAZE_W + x) as usize];

                let start_x = (x as f64 * scale_x).round() as i32;
                let end_x = ((x + 1) as f64 * scale_x).round() as i32;
                let start_y = (y as f64 * scale_y).round() as i32;
                let end_y = ((y + 1) as f64 * scale_y).round() as i32;

                for draw_y in start_y..end_y {
                    for draw_x in start_x..end_x {
                        if draw_x >= 0
                            && draw_x < self.width as i32
                            && draw_y >= 0
                            && draw_y < self.height as i32
                        {
                            if cell == 3 {
                                let mid_x = (start_x + end_x) / 2;
                                let mid_y = (start_y + end_y) / 2;

                                let mut should_draw = false;

                                if draw_x == mid_x && draw_y == mid_y {
                                    should_draw = true;
                                } else if draw_x == mid_x {
                                    if draw_y < mid_y
                                        && y > 0
                                        && self.dots[((y - 1) * MAZE_W + x) as usize] == 3
                                    {
                                        should_draw = true;
                                    }
                                    if draw_y > mid_y
                                        && y < MAZE_H - 1
                                        && self.dots[((y + 1) * MAZE_W + x) as usize] == 3
                                    {
                                        should_draw = true;
                                    }
                                } else if draw_y == mid_y {
                                    if draw_x < mid_x
                                        && x > 0
                                        && self.dots[(y * MAZE_W + (x - 1)) as usize] == 3
                                    {
                                        should_draw = true;
                                    }
                                    if draw_x > mid_x
                                        && x < MAZE_W - 1
                                        && self.dots[(y * MAZE_W + (x + 1)) as usize] == 3
                                    {
                                        should_draw = true;
                                    }
                                }

                                if should_draw {
                                    let color = crossterm::style::Color::Rgb {
                                        r: 33,
                                        g: 33,
                                        b: 255,
                                    };
                                    buffer.set(
                                        draw_x as u16,
                                        draw_y as u16,
                                        wall_char,
                                        color,
                                        crossterm::style::Color::Reset,
                                    );
                                }
                            } else if cell == 1 {
                                let mid_x = (start_x + end_x) / 2;
                                let mid_y = (start_y + end_y) / 2;
                                if draw_x == mid_x && draw_y == mid_y {
                                    let color = crossterm::style::Color::Rgb {
                                        r: 255,
                                        g: 184,
                                        b: 174,
                                    };
                                    buffer.set(
                                        draw_x as u16,
                                        draw_y as u16,
                                        dot_char,
                                        color,
                                        crossterm::style::Color::Reset,
                                    );
                                }
                            } else if cell == 2 {
                                let mid_x = (start_x + end_x) / 2;
                                let mid_y = (start_y + end_y) / 2;
                                if draw_x == mid_x && draw_y == mid_y {
                                    let color = crossterm::style::Color::Rgb {
                                        r: 255,
                                        g: 184,
                                        b: 174,
                                    };
                                    buffer.set(
                                        draw_x as u16,
                                        draw_y as u16,
                                        big_dot_char,
                                        color,
                                        crossterm::style::Color::Reset,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        for (i, entity) in self.entities.iter().enumerate() {
            let start_x = (entity.x * scale_x).round() as i32;
            let end_x = ((entity.x + 1.0) * scale_x).round() as i32;
            let start_y = (entity.y * scale_y).round() as i32;
            let end_y = ((entity.y + 1.0) * scale_y).round() as i32;

            for draw_y in start_y..end_y {
                for draw_x in start_x..end_x {
                    if draw_x >= 0
                        && draw_x < self.width as i32
                        && draw_y >= 0
                        && draw_y < self.height as i32
                    {
                        if entity.is_pacman {
                            let color = crossterm::style::Color::Rgb {
                                r: 255,
                                g: 255,
                                b: 0,
                            };
                            buffer.set(
                                draw_x as u16,
                                draw_y as u16,
                                pacman_char,
                                color,
                                crossterm::style::Color::Reset,
                            );
                        } else {
                            let color = match i % 4 {
                                0 => crossterm::style::Color::Rgb { r: 255, g: 0, b: 0 },
                                1 => crossterm::style::Color::Rgb {
                                    r: 255,
                                    g: 184,
                                    b: 255,
                                },
                                2 => crossterm::style::Color::Rgb {
                                    r: 0,
                                    g: 255,
                                    b: 255,
                                },
                                _ => crossterm::style::Color::Rgb {
                                    r: 255,
                                    g: 184,
                                    b: 82,
                                },
                            };
                            buffer.set(
                                draw_x as u16,
                                draw_y as u16,
                                ghost_char,
                                color,
                                crossterm::style::Color::Reset,
                            );
                        }
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
