use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemePalette;
use crate::visualizer::Visualizer;
use crossterm::style::Color;
use noise::{NoiseFn, Perlin};
use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
enum Biome {
    Ocean,
    Plains,
    Forest,
    Desert,
    Snow,
}

pub struct MinecraftVisualizer {
    width: u16,
    height: u16,
    time: f64,
    cam_x: f64,
    cam_y: f64,
    cam_z: f64,
    yaw: f64,
    pitch: f64,
    speed: f64,

    perlin_elev: Perlin,
    perlin_temp: Perlin,
    perlin_moist: Perlin,
}

impl MinecraftVisualizer {
    pub fn new(width: u16, height: u16, _palette: ThemePalette, _charset: CharSet) -> Self {
        Self {
            width,
            height,
            time: 0.0,
            cam_x: 0.0,
            cam_y: 0.0,
            cam_z: 60.0,
            yaw: 0.0,
            pitch: -0.6,
            speed: 2.0,

            perlin_elev: Perlin::new(rand::random::<u32>()),
            perlin_temp: Perlin::new(rand::random::<u32>()),
            perlin_moist: Perlin::new(rand::random::<u32>()),
        }
    }

    fn get_biome(&self, x_f: f64, y_f: f64) -> Biome {
        let temp = self.perlin_temp.get([x_f * 0.02, y_f * 0.02]);
        let moist = self.perlin_moist.get([x_f * 0.02, y_f * 0.02]);

        let elev = 15.0
            + (x_f * 0.03).sin() * 10.0
            + (y_f * 0.03).cos() * 10.0
            + (x_f * 0.1).sin() * 4.0
            + (y_f * 0.1).cos() * 4.0
            + (x_f * 0.3).sin() * 1.0;

        if elev < 13.0 {
            return Biome::Ocean;
        }
        if elev > 30.0 {
            return Biome::Snow;
        }

        if temp > 0.1 && moist < 0.1 {
            Biome::Desert
        } else if moist > 0.0 {
            Biome::Forest
        } else {
            Biome::Plains
        }
    }

    fn get_terrain_height_internal(&self, x_f: f64, y_f: f64, _biome: Biome) -> f64 {
        15.0 + (x_f * 0.03).sin() * 10.0
            + (y_f * 0.03).cos() * 10.0
            + (x_f * 0.1).sin() * 4.0
            + (y_f * 0.1).cos() * 4.0
            + (x_f * 0.3).sin() * 1.0
    }

    fn get_terrain_height(&self, x_f: f64, y_f: f64) -> f64 {
        let biome = self.get_biome(x_f, y_f);
        self.get_terrain_height_internal(x_f, y_f, biome)
    }

    fn get_block(
        &self,
        x: i32,
        y: i32,
        z: i32,
        terrain_cache: &[(f64, Biome)],
        min_x: i32,
        min_y: i32,
    ) -> u8 {
        if z < 0 {
            return 4;
        }

        let x_f = x as f64;
        let y_f = y as f64;

        if z >= 70 && z <= 72 {
            let drift = self.time * 8.0;
            let cloud_scale = 12.0;
            let cx = ((x_f - drift) / cloud_scale).floor();
            let cy = (y_f / cloud_scale).floor();

            let cloud_val = self
                .perlin_moist
                .get([cx * 0.15 + 999.0, cy * 0.15 + 999.0]);

            if cloud_val > 0.15 {
                return 7;
            }
        }

        let hx = x - min_x;
        let hy = y - min_y;
        let (height, biome) = if hx >= 0 && hx < 220 && hy >= 0 && hy < 220 {
            terrain_cache[(hy * 220 + hx) as usize]
        } else {
            let b = self.get_biome(x_f, y_f);
            (self.get_terrain_height_internal(x_f, y_f, b), b)
        };

        let water_level = 10;

        if (z as f64) > height {
            if z <= water_level {
                if biome == Biome::Snow && z == water_level {
                    return 13;
                }
                return 2;
            }

            if (z as f64) < height + 10.0 {
                let hash =
                    (x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)).wrapping_mul(83492791);

                let is_tree_spot = hash % 100
                    < match biome {
                        Biome::Forest => 3,
                        Biome::Snow => 1,
                        Biome::Desert => 1,
                        Biome::Plains => 1,
                        Biome::Ocean => 0,
                    }
                    && height > water_level as f64;

                if is_tree_spot {
                    let base_z = height as i32 + 1;

                    if biome == Biome::Desert {
                        if z >= base_z && z < base_z + 3 {
                            return 9;
                        }
                    } else if biome == Biome::Snow {
                        if z >= base_z && z < base_z + 6 {
                            return 5;
                        }
                    } else {
                        if z >= base_z && z < base_z + 5 {
                            if hash % 2 == 0 {
                                return 11;
                            }
                            return 5;
                        }
                    }
                }

                for dx in -2..=2 {
                    for dy in -2..=2 {
                        let nx = x + dx;
                        let ny = y + dy;
                        let n_hash = (nx.wrapping_mul(73856093) ^ ny.wrapping_mul(19349663))
                            .wrapping_mul(83492791);
                        let nhx = nx - min_x;
                        let nhy = ny - min_y;

                        let (n_height, n_biome) = if nhx >= 0 && nhx < 220 && nhy >= 0 && nhy < 220
                        {
                            terrain_cache[(nhy * 220 + nhx) as usize]
                        } else {
                            let b = self.get_biome(nx as f64, ny as f64);
                            (self.get_terrain_height_internal(nx as f64, ny as f64, b), b)
                        };

                        let n_is_tree = n_hash % 100
                            < match n_biome {
                                Biome::Forest => 3,
                                Biome::Snow => 1,
                                Biome::Desert => 1,
                                Biome::Plains => 1,
                                Biome::Ocean => 0,
                            };

                        if n_is_tree && n_biome != Biome::Desert {
                            if n_height > water_level as f64 {
                                let top_z = if n_biome == Biome::Snow {
                                    n_height as i32 + 6
                                } else {
                                    n_height as i32 + 5
                                };

                                if n_biome == Biome::Snow {
                                    let dz = top_z - z;
                                    if dz >= 0 && dz < 5 {
                                        let radius = dz as f64 * 0.8;
                                        let dist = ((dx * dx + dy * dy) as f64).sqrt();
                                        if dist <= radius {
                                            return 10;
                                        }
                                    }
                                } else {
                                    let dist =
                                        (dx * dx + dy * dy + (z - top_z) * (z - top_z)) as f64;
                                    if dist < 8.0 {
                                        if n_hash % 2 == 0 {
                                            return 12;
                                        }
                                        return 6;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            return 0;
        }

        let dist_below = height - z as f64;

        if dist_below < 1.0 {
            if height <= (water_level + 2) as f64 && biome != Biome::Snow && biome != Biome::Forest
            {
                return 3;
            }
            if biome == Biome::Snow {
                return 8;
            }
            if biome == Biome::Desert {
                return 3;
            }
            if height > 32.0 {
                return 4;
            }
            return 1;
        }

        if dist_below < 4.0 {
            if height <= (water_level + 2) as f64 && biome != Biome::Snow && biome != Biome::Forest
            {
                return 3;
            }
            if biome == Biome::Desert {
                return 3;
            }
            if biome == Biome::Snow {
                return 4;
            }
            return 14;
        }

        4
    }
}

impl Visualizer for MinecraftVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = delta_time * self.speed;
        self.time += dt;

        self.cam_y += dt * 4.0;
        self.cam_x += dt * 1.0;

        let mut terrain_z = self.get_terrain_height(self.cam_x, self.cam_y);
        let water_level = 10.0;
        if terrain_z < water_level {
            terrain_z = water_level;
        }

        let desired_z = terrain_z + 20.0;
        self.cam_z += (desired_z - self.cam_z) * dt * 2.0;

        self.yaw = (self.time * 0.05).sin() * 0.4;
        self.pitch = -0.4 + (self.time * 0.1).cos() * 0.1;
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
        }
        buffer.clear();

        let day_cycle = (self.time * 0.008) % (std::f64::consts::PI * 2.0);
        let sun_height = day_cycle.sin();

        let mut sky_r = 135.0;
        let mut sky_g = 206.0;
        let mut sky_b = 235.0;

        if sun_height > 0.2 {
        } else if sun_height > -0.2 {
            let blend = (sun_height + 0.2) / 0.4;
            sky_r = 25.0 * (1.0 - blend) + 135.0 * blend;
            sky_g = 15.0 * (1.0 - blend) + 206.0 * blend;
            sky_b = 35.0 * (1.0 - blend) + 235.0 * blend;

            let sunset_peak = 1.0 - ((blend - 0.5) * 2.0).abs();
            sky_r += sunset_peak * 120.0;
            sky_g += sunset_peak * 40.0;
            sky_b -= sunset_peak * 50.0;
        } else {
            sky_r = 10.0;
            sky_g = 15.0;
            sky_b = 30.0;
        }

        sky_r = sky_r.clamp(0.0, 255.0);
        sky_g = sky_g.clamp(0.0, 255.0);
        sky_b = sky_b.clamp(0.0, 255.0);

        let dir_x = self.yaw.sin() * self.pitch.cos();
        let dir_y = self.yaw.cos() * self.pitch.cos();
        let dir_z = self.pitch.sin();

        let right_x = self.yaw.cos();
        let right_y = -self.yaw.sin();
        let right_z = 0.0;

        let up_x = right_y * dir_z - right_z * dir_y;
        let up_y = right_z * dir_x - right_x * dir_z;
        let up_z = right_x * dir_y - right_y * dir_x;

        let fov = 1.0;

        let min_x = (self.cam_x.floor() as i32) - 110;
        let min_y = (self.cam_y.floor() as i32) - 110;
        let mut terrain_cache = vec![(0.0, Biome::Plains); 220 * 220];
        for hy in 0..220 {
            for hx in 0..220 {
                let cx = (min_x + hx) as f64;
                let cy = (min_y + hy) as f64;
                let b = self.get_biome(cx, cy);
                let h = self.get_terrain_height_internal(cx, cy, b);
                terrain_cache[(hy * 220 + hx) as usize] = (h, b);
            }
        }

        for sy in 0..self.height {
            for sx in 0..self.width {
                let ndc_x = (2.0 * sx as f64 / self.width as f64 - 1.0)
                    * (self.width as f64 / self.height as f64)
                    * 0.5;
                let ndc_y = 1.0 - 2.0 * sy as f64 / self.height as f64;

                let mut rx = dir_x * fov + right_x * ndc_x + up_x * ndc_y;
                let mut ry = dir_y * fov + right_y * ndc_x + up_y * ndc_y;
                let mut rz = dir_z * fov + right_z * ndc_x + up_z * ndc_y;

                let len = (rx * rx + ry * ry + rz * rz).sqrt();
                rx /= len;
                ry /= len;
                rz /= len;

                let mut map_x = self.cam_x.floor() as i32;
                let mut map_y = self.cam_y.floor() as i32;
                let mut map_z = self.cam_z.floor() as i32;

                let delta_dist_x = if rx == 0.0 { 1e30 } else { (1.0 / rx).abs() };
                let delta_dist_y = if ry == 0.0 { 1e30 } else { (1.0 / ry).abs() };
                let delta_dist_z = if rz == 0.0 { 1e30 } else { (1.0 / rz).abs() };

                let step_x = if rx < 0.0 { -1 } else { 1 };
                let step_y = if ry < 0.0 { -1 } else { 1 };
                let step_z = if rz < 0.0 { -1 } else { 1 };

                let mut side_dist_x = if rx < 0.0 {
                    (self.cam_x - map_x as f64) * delta_dist_x
                } else {
                    (map_x as f64 + 1.0 - self.cam_x) * delta_dist_x
                };
                let mut side_dist_y = if ry < 0.0 {
                    (self.cam_y - map_y as f64) * delta_dist_y
                } else {
                    (map_y as f64 + 1.0 - self.cam_y) * delta_dist_y
                };
                let mut side_dist_z = if rz < 0.0 {
                    (self.cam_z - map_z as f64) * delta_dist_z
                } else {
                    (map_z as f64 + 1.0 - self.cam_z) * delta_dist_z
                };

                let mut hit = 0;
                let mut side = 0;

                let max_steps = 100;
                for _ in 0..max_steps {
                    if side_dist_x < side_dist_y {
                        if side_dist_x < side_dist_z {
                            side_dist_x += delta_dist_x;
                            map_x += step_x;
                            side = 0;
                        } else {
                            side_dist_z += delta_dist_z;
                            map_z += step_z;
                            side = 2;
                        }
                    } else {
                        if side_dist_y < side_dist_z {
                            side_dist_y += delta_dist_y;
                            map_y += step_y;
                            side = 1;
                        } else {
                            side_dist_z += delta_dist_z;
                            map_z += step_z;
                            side = 2;
                        }
                    }

                    let block = self.get_block(map_x, map_y, map_z, &terrain_cache, min_x, min_y);
                    if block != 0 {
                        hit = block;
                        break;
                    }
                }

                if hit == 0 {
                    let mut draw_char = ' ';
                    let mut fg = Color::Reset;
                    if sun_height < -0.2 {
                        let star_hash =
                            (sx.wrapping_mul(9123) ^ sy.wrapping_mul(5829)).wrapping_mul(1239);
                        if star_hash % 100 < 2 {
                            draw_char = '.';
                            fg = Color::Rgb {
                                r: 200,
                                g: 200,
                                b: 255,
                            };
                        }
                    }
                    buffer.set(
                        sx as u16,
                        sy as u16,
                        draw_char,
                        fg,
                        Color::Rgb {
                            r: sky_r as u8,
                            g: sky_g as u8,
                            b: sky_b as u8,
                        },
                    );
                    continue;
                }

                let perp_wall_dist = match side {
                    0 => side_dist_x - delta_dist_x,
                    1 => side_dist_y - delta_dist_y,
                    _ => side_dist_z - delta_dist_z,
                };

                let char_to_draw = match hit {
                    1 => '█',
                    2 => {
                        if ((map_x as f64 * 0.5 + self.time * 2.0).sin()) > 0.0 {
                            '~'
                        } else {
                            '≈'
                        }
                    }
                    3 => '█',
                    4 => '█',
                    5 => '█',
                    6 => '▒',
                    7 => '█',
                    8 => '█',
                    9 => '█',
                    10 => '▒',
                    11 => '█',
                    12 => '▒',
                    13 => '█',
                    14 => '█',
                    _ => '█',
                };

                let (mut r, mut g, mut b) = match hit {
                    1 => (85, 170, 85),
                    2 => (50, 100, 220),
                    3 => (240, 220, 120),
                    4 => (120, 120, 120),
                    5 => (110, 80, 50),
                    6 => (40, 130, 40),
                    7 => (255, 255, 255),
                    8 => (240, 250, 255),
                    9 => (30, 160, 30),
                    10 => (20, 90, 40),
                    11 => (210, 210, 200),
                    12 => (120, 180, 80),
                    13 => (150, 200, 255),
                    14 => (139, 69, 19),
                    _ => (0, 0, 0),
                };

                let light_dir_x = day_cycle.cos();
                let light_dir_y = day_cycle.sin();

                let mut light_intensity = 1.0;

                let mut global_dim = if sun_height > 0.0 {
                    1.0
                } else {
                    0.2 + (sun_height + 1.0) * 0.4
                };
                if global_dim > 1.0 {
                    global_dim = 1.0;
                }
                if global_dim < 0.2 {
                    global_dim = 0.2;
                }

                if hit != 7 {
                    if side == 0 {
                        if (map_x as f64 - self.cam_x) * light_dir_x > 0.0 {
                            light_intensity = 0.6;
                        } else {
                            light_intensity = 0.9;
                        }
                    } else if side == 1 {
                        if (map_y as f64 - self.cam_y) * light_dir_y > 0.0 {
                            light_intensity = 0.5;
                        } else {
                            light_intensity = 0.8;
                        }
                    } else {
                        light_intensity = 1.0;
                    }
                }

                r = (r as f32 * light_intensity as f32 * global_dim as f32) as u8;
                g = (g as f32 * light_intensity as f32 * global_dim as f32) as u8;
                b = (b as f32 * light_intensity as f32 * global_dim as f32) as u8;

                let fog_factor = (1.0 - perp_wall_dist / max_steps as f64).clamp(0.0, 1.0);
                let final_r = (r as f64 * fog_factor + sky_r * (1.0 - fog_factor)) as u8;
                let final_g = (g as f64 * fog_factor + sky_g * (1.0 - fog_factor)) as u8;
                let final_b = (b as f64 * fog_factor + sky_b * (1.0 - fog_factor)) as u8;

                buffer.set(
                    sx as u16,
                    sy as u16,
                    char_to_draw,
                    Color::Rgb {
                        r: final_r,
                        g: final_g,
                        b: final_b,
                    },
                    Color::Reset,
                );
            }
        }
    }

    fn set_palette(&mut self, _palette: ThemePalette) {}
    fn set_charset(&mut self, _charset: CharSet) {}

    fn on_scroll(&mut self, delta: i32) {
        self.speed += delta as f64 * 1.0;
        self.speed = self.speed.clamp(1.0, 50.0);
    }
}
