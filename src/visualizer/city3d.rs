use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use rand::Rng;

const MAP_WIDTH: usize = 128;
const MAP_HEIGHT: usize = 128;

const TREE_SPRITE: [&str; 14] = [
    "      .      ",
    "     / \\     ",
    "    /   \\    ",
    "   /     \\   ",
    "  /_______\\  ",
    "    /   \\    ",
    "   /     \\   ",
    "  /       \\  ",
    " /_________\\ ",
    "   /     \\   ",
    "  /       \\  ",
    " /_________\\ ",
    "     |||     ",
    "     |||     ",
];

#[derive(Clone, Copy)]
struct Sprite {
    x: f64,
    y: f64,
    kind: u8,
}

pub struct City3DVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,

    map: [[u8; MAP_HEIGHT]; MAP_WIDTH],
    sprites: Vec<Sprite>,
    z_buffer: Vec<f64>,

    pos_x: f64,
    pos_y: f64,
    dir_angle: f64,

    move_speed: f64,
    turn_speed: f64,

    speed_multiplier: f64,
}

impl City3DVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            width,
            height,
            palette,
            charset,
            map: [[1; MAP_HEIGHT]; MAP_WIDTH],
            sprites: Vec::new(),
            z_buffer: vec![0.0; width as usize],

            pos_x: 64.5,
            pos_y: 64.5,
            dir_angle: std::f64::consts::PI / 2.0,

            move_speed: 4.0,
            turn_speed: 3.0,

            speed_multiplier: 1.0,
        };
        vis.generate_map();
        vis
    }

    fn generate_map(&mut self) {
        let mut rng = rand::thread_rng();

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                self.map[x][y] = 1;
            }
        }

        for x in (40..88).step_by(4) {
            for y in 40..88 {
                self.map[x][y] = 0;
            }
        }
        for y in (40..88).step_by(4) {
            for x in 40..88 {
                self.map[x][y] = 0;
            }
        }

        let center_x = 64.0;
        let center_y = 64.0;
        let base_radius = 45.0;

        let steps = 1000;
        for i in 0..steps {
            let angle = (i as f64 / steps as f64) * std::f64::consts::PI * 2.0;
            let radius = base_radius + (angle * 6.0).sin() * 8.0 + (angle * 3.0).cos() * 5.0;

            let cx = center_x + angle.cos() * radius;
            let cy = center_y + angle.sin() * radius;

            let road_width = 3;
            for dx in -road_width..=road_width {
                for dy in -road_width..=road_width {
                    if dx * dx + dy * dy <= road_width * road_width {
                        let px = (cx + dx as f64).round() as i32;
                        let py = (cy + dy as f64).round() as i32;
                        if px > 0
                            && px < MAP_WIDTH as i32 - 1
                            && py > 0
                            && py < MAP_HEIGHT as i32 - 1
                        {
                            self.map[px as usize][py as usize] = 0;
                        }
                    }
                }
            }
        }

        for x in 64..=66 {
            for y in 20..108 {
                if self.map[x][y] == 1 {
                    self.map[x][y] = 0;
                }
            }
        }
        for y in 64..=66 {
            for x in 20..108 {
                if self.map[x][y] == 1 {
                    self.map[x][y] = 0;
                }
            }
        }

        self.sprites.clear();
        for x in 1..MAP_WIDTH - 1 {
            for y in 1..MAP_HEIGHT - 1 {
                if self.map[x][y] == 1 {
                    if x > 38 && x < 90 && y > 38 && y < 90 {
                        self.map[x][y] = rng.gen_range(2..=4);
                    }
                } else if self.map[x][y] == 0 {
                    let mut adjacent_walls = 0;
                    if self.map[x - 1][y] > 0 {
                        adjacent_walls += 1;
                    }
                    if self.map[x + 1][y] > 0 {
                        adjacent_walls += 1;
                    }
                    if self.map[x][y - 1] > 0 {
                        adjacent_walls += 1;
                    }
                    if self.map[x][y + 1] > 0 {
                        adjacent_walls += 1;
                    }

                    if adjacent_walls > 0 {
                        let spawn_chance = if x > 38 && x < 90 && y > 38 && y < 90 {
                            0.1
                        } else {
                            0.8
                        };
                        if rng.gen_bool(spawn_chance) {
                            self.sprites.push(Sprite {
                                x: x as f64 + 0.5 + rng.gen_range(-0.3..0.3),
                                y: y as f64 + 0.5 + rng.gen_range(-0.3..0.3),
                                kind: 0,
                            });
                        }
                    }
                }
            }
        }

        self.pos_x = 64.5;
        self.pos_y = 64.5;
        self.dir_angle = std::f64::consts::PI / 2.0;
    }

    fn cast_ray(&self, angle: f64) -> f64 {
        let dir_x = angle.cos();
        let dir_y = angle.sin();
        let mut map_x = self.pos_x as i32;
        let mut map_y = self.pos_y as i32;

        let delta_dist_x = if dir_x == 0.0 {
            1e30
        } else {
            (1.0 / dir_x).abs()
        };
        let delta_dist_y = if dir_y == 0.0 {
            1e30
        } else {
            (1.0 / dir_y).abs()
        };

        let mut side_dist_x;
        let mut side_dist_y;
        let step_x: i32;
        let step_y: i32;

        if dir_x < 0.0 {
            step_x = -1;
            side_dist_x = (self.pos_x - map_x as f64) * delta_dist_x;
        } else {
            step_x = 1;
            side_dist_x = (map_x as f64 + 1.0 - self.pos_x) * delta_dist_x;
        }

        if dir_y < 0.0 {
            step_y = -1;
            side_dist_y = (self.pos_y - map_y as f64) * delta_dist_y;
        } else {
            step_y = 1;
            side_dist_y = (map_y as f64 + 1.0 - self.pos_y) * delta_dist_y;
        }

        let mut hit = false;
        let mut dist = 0.0;
        let max_dist = 20.0;

        while !hit && dist < max_dist {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                dist = side_dist_x - delta_dist_x;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                dist = side_dist_y - delta_dist_y;
            }

            if map_x < 0 || map_x >= MAP_WIDTH as i32 || map_y < 0 || map_y >= MAP_HEIGHT as i32 {
                hit = true;
            } else if self.map[map_x as usize][map_y as usize] > 0 {
                hit = true;
            }
        }
        dist
    }
}

impl Visualizer for City3DVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = delta_time * self.speed_multiplier;
        let _rng = rand::thread_rng();

        let whisker_angle = 0.7;
        let dist_center = self.cast_ray(self.dir_angle);
        let dist_left = self.cast_ray(self.dir_angle - whisker_angle);
        let dist_right = self.cast_ray(self.dir_angle + whisker_angle);

        let mut turn = 0.0;
        let mut speed = self.move_speed;

        if dist_center < 5.0 {
            speed = self.move_speed * 0.4;

            if dist_left < 2.0 && dist_right < 2.0 {
                turn = self.turn_speed * 2.5;
            } else {
                let diff = dist_right - dist_left;
                turn = diff.signum() * self.turn_speed * 2.0;
            }
        } else {
            let diff = dist_right - dist_left;

            if dist_left < 8.0 || dist_right < 8.0 {
                turn = diff * 0.4;
                turn = turn.clamp(-self.turn_speed * 0.8, self.turn_speed * 0.8);
            }
        }

        self.dir_angle += turn * dt;

        let dir_x = self.dir_angle.cos();
        let dir_y = self.dir_angle.sin();

        let new_x = self.pos_x + dir_x * speed * dt;
        let new_y = self.pos_y + dir_y * speed * dt;

        let margin = 0.4;
        let look_x = (new_x + dir_x * margin) as usize;
        let look_y = (new_y + dir_y * margin) as usize;

        if look_x < MAP_WIDTH && look_y < MAP_HEIGHT && self.map[look_x][look_y] == 0 {
            self.pos_x = new_x;
            self.pos_y = new_y;
        } else {
            self.dir_angle += std::f64::consts::PI / 2.0;
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.z_buffer = vec![0.0; self.width as usize];
        }

        buffer.clear();

        for y in 0..self.height / 2 {
            let grad = y as f64 / (self.height as f64 / 2.0);
            let color = interpolate_gradient(&self.palette, (grad * 0.3) as f32);
            for x in 0..self.width {
                buffer.set(x, y, ' ', color, color);
            }
        }

        let dir_x = self.dir_angle.cos();
        let dir_y = self.dir_angle.sin();
        let plane_x = -dir_y * 0.66;
        let plane_y = dir_x * 0.66;

        for y in self.height / 2..self.height {
            let p = y as f64 - self.height as f64 / 2.0;
            let cam_z = self.height as f64 / 2.0;
            let row_distance = if p == 0.0 { 1e30 } else { cam_z / p };

            let ray_dir_x0 = dir_x - plane_x;
            let ray_dir_y0 = dir_y - plane_y;
            let ray_dir_x1 = dir_x + plane_x;
            let ray_dir_y1 = dir_y + plane_y;

            let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / self.width as f64;
            let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / self.width as f64;

            let mut floor_x = self.pos_x + row_distance * ray_dir_x0;
            let mut floor_y = self.pos_y + row_distance * ray_dir_y0;

            for x in 0..self.width {
                let cell_x = floor_x as i32;
                let cell_y = floor_y as i32;

                let tx = floor_x - cell_x as f64;
                let ty = floor_y - cell_y as f64;

                let is_road = cell_x >= 0
                    && cell_x < MAP_WIDTH as i32
                    && cell_y >= 0
                    && cell_y < MAP_HEIGHT as i32
                    && self.map[cell_x as usize][cell_y as usize] == 0;
                let dist_fade = (1.0 - (row_distance / 25.0)).clamp(0.0, 1.0);

                let mut color = interpolate_gradient(&self.palette, (0.7 + dist_fade * 0.3) as f32);
                let mut char = ' ';

                if is_road {
                    color = interpolate_gradient(&self.palette, (0.2 + dist_fade * 0.2) as f32);

                    if tx > 0.46 && tx < 0.54 && (ty % 0.5) < 0.25 {
                        color = interpolate_gradient(&self.palette, 0.9);
                        char = '│';
                    } else if ty > 0.46 && ty < 0.54 && (tx % 0.5) < 0.25 {
                        color = interpolate_gradient(&self.palette, 0.9);
                        char = '─';
                    } else {
                        char = ' ';
                    }
                } else {
                    char = ' ';
                }

                buffer.set(x, y, char, color, crossterm::style::Color::Reset);

                floor_x += floor_step_x;
                floor_y += floor_step_y;
            }
        }

        for x in 0..self.width {
            let camera_x = 2.0 * x as f64 / self.width as f64 - 1.0;
            let ray_dir_x = dir_x + plane_x * camera_x;
            let ray_dir_y = dir_y + plane_y * camera_x;

            let mut map_x = self.pos_x as i32;
            let mut map_y = self.pos_y as i32;

            let delta_dist_x = if ray_dir_x == 0.0 {
                1e30
            } else {
                (1.0 / ray_dir_x).abs()
            };
            let delta_dist_y = if ray_dir_y == 0.0 {
                1e30
            } else {
                (1.0 / ray_dir_y).abs()
            };

            let mut side_dist_x;
            let mut side_dist_y;

            let step_x: i32;
            let step_y: i32;

            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (self.pos_x - map_x as f64) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f64 + 1.0 - self.pos_x) * delta_dist_x;
            }

            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (self.pos_y - map_y as f64) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f64 + 1.0 - self.pos_y) * delta_dist_y;
            }

            let mut hit = false;
            let mut side = 0;

            while !hit {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = 1;
                }

                if map_x < 0 || map_x >= MAP_WIDTH as i32 || map_y < 0 || map_y >= MAP_HEIGHT as i32
                {
                    hit = true;
                } else if self.map[map_x as usize][map_y as usize] > 0 {
                    hit = true;
                }
            }

            let perp_wall_dist = if side == 0 {
                side_dist_x - delta_dist_x
            } else {
                side_dist_y - delta_dist_y
            };

            self.z_buffer[x as usize] = perp_wall_dist;

            let line_height = (self.height as f64 / perp_wall_dist.max(0.1) * 1.5) as i32;

            let mut draw_start = -line_height / 2 + self.height as i32 / 2;
            if draw_start < 0 {
                draw_start = 0;
            }
            let mut draw_end = line_height / 2 + self.height as i32 / 2;
            if draw_end >= self.height as i32 {
                draw_end = self.height as i32 - 1;
            }

            let wall_type = if map_x >= 0
                && map_x < MAP_WIDTH as i32
                && map_y >= 0
                && map_y < MAP_HEIGHT as i32
            {
                self.map[map_x as usize][map_y as usize]
            } else {
                1
            };

            let base_gradient = match wall_type {
                1 => 0.1,
                2 => 0.3,
                3 => 0.6,
                4 => 0.8,
                _ => 0.5,
            };

            let color_mod = if side == 1 { 0.1 } else { 0.0 };

            let distance_fade = (perp_wall_dist / 15.0).clamp(0.0, 1.0);

            let gradient_val = (base_gradient - color_mod - (distance_fade * 0.3)).clamp(0.0, 1.0);

            let color = interpolate_gradient(&self.palette, gradient_val as f32);

            for y in draw_start..=draw_end {
                let tex_y = ((y - draw_start) as f64 * 3.0 / line_height as f64).floor() as i32;
                let c = match tex_y {
                    0 | 3 => '▓',
                    1 => '▒',
                    2 => '░',
                    _ => '█',
                };

                buffer.set(x, y as u16, c, color, crossterm::style::Color::Reset);
            }
        }

        let mut sorted_sprites = self.sprites.clone();
        let px = self.pos_x;
        let py = self.pos_y;
        sorted_sprites.sort_by(|a, b| {
            let dist_a = (px - a.x).powi(2) + (py - a.y).powi(2);
            let dist_b = (px - b.x).powi(2) + (py - b.y).powi(2);
            dist_b
                .partial_cmp(&dist_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let chars_len = self.charset.chars.len();
        let _tree_char = if chars_len > 0 {
            self.charset.chars[chars_len / 2]
        } else {
            '♠'
        };

        for sprite in sorted_sprites {
            let sprite_x = sprite.x - self.pos_x;
            let sprite_y = sprite.y - self.pos_y;

            let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
            let transform_x = inv_det * (dir_y * sprite_x - dir_x * sprite_y);
            let transform_y = inv_det * (-plane_y * sprite_x + plane_x * sprite_y);

            if transform_y <= 0.0 {
                continue;
            }

            let sprite_screen_x =
                ((self.width as f64 / 2.0) * (1.0 + transform_x / transform_y)) as i32;

            let sprite_height = (self.height as f64 / transform_y * 1.5).abs() as i32;

            let mut draw_start_y = -sprite_height / 2 + self.height as i32 / 2;
            if draw_start_y < 0 {
                draw_start_y = 0;
            }
            let mut draw_end_y = sprite_height / 2 + self.height as i32 / 2;
            if draw_end_y >= self.height as i32 {
                draw_end_y = self.height as i32 - 1;
            }

            let sprite_width = (self.width as f64 / transform_y * 1.5).abs() as i32;
            let mut draw_start_x = -sprite_width / 2 + sprite_screen_x;
            if draw_start_x < 0 {
                draw_start_x = 0;
            }
            let mut draw_end_x = sprite_width / 2 + sprite_screen_x;
            if draw_end_x >= self.width as i32 {
                draw_end_x = self.width as i32 - 1;
            }

            let tree_color = interpolate_gradient(&self.palette, 0.4);

            for stripe in draw_start_x..draw_end_x {
                if stripe >= 0
                    && stripe < self.width as i32
                    && transform_y < self.z_buffer[stripe as usize]
                {
                    let local_x = (stripe - (-sprite_width / 2 + sprite_screen_x)) as f64
                        / sprite_width as f64;

                    for y in draw_start_y..=draw_end_y {
                        let local_y = (y - (-sprite_height / 2 + self.height as i32 / 2)) as f64
                            / sprite_height as f64;

                        let px = (local_x * 13.0) as usize;
                        let py = (local_y * 14.0) as usize;
                        if px < 13 && py < 14 {
                            let c = TREE_SPRITE[py].chars().nth(px).unwrap_or(' ');
                            if c != ' ' {
                                buffer.set(
                                    stripe as u16,
                                    y as u16,
                                    c,
                                    tree_color,
                                    crossterm::style::Color::Reset,
                                );
                            }
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
