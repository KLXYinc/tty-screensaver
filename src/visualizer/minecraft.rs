use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemePalette;
use crate::visualizer::Visualizer;
use crossterm::style::Color;
use noise::{NoiseFn, Perlin};
const AIR: u8 = 0;
const GRASS: u8 = 1;
const WATER_S: u8 = 2;
const SAND: u8 = 3;
const STONE: u8 = 4;
const OAK_WOOD: u8 = 5;
const OAK_LEAF: u8 = 6;
const CLOUD: u8 = 7;
const SNOW_BLOCK: u8 = 8;
const JUNGLE_LEAF: u8 = 9;
const SPRUCE_LEAF: u8 = 10;
const DIRT: u8 = 14;
const GRAVEL: u8 = 15;
const SAVANNA_GRASS: u8 = 16;
const ACACIA_WOOD: u8 = 17;
const ACACIA_LEAF: u8 = 18;
const WATER_D: u8 = 19;
const BEDROCK: u8 = 20;
const SWAMP_GRASS: u8 = 21;
const WATER_LEVEL: i32 = 10;
const CACHE_W: i32 = 384;
const BIOME_OCEAN: u8 = 0;
const BIOME_PLAINS: u8 = 1;
const BIOME_FOREST: u8 = 2;
const BIOME_DESERT: u8 = 3;
const BIOME_SNOW: u8 = 4;
const BIOME_JUNGLE: u8 = 5;
const BIOME_SAVANNA: u8 = 6;
const BIOME_MOUNTAIN: u8 = 7;
const BIOME_SWAMP: u8 = 8;
#[derive(Clone, Copy, Default)]
struct CacheCell {
    terrain_h: f32,
    canopy_top: f32,
    trunk_top: f32,
    biome: u8,
}
#[inline]
fn block_color(hit: u8) -> (u8, u8, u8) {
    match hit {
        GRASS => (89, 172, 60),
        SAVANNA_GRASS => (168, 168, 72),
        SWAMP_GRASS => (62, 120, 52),
        WATER_S => (45, 128, 220),
        WATER_D => (18, 58, 160),
        SAND => (218, 196, 118),
        STONE => (122, 122, 122),
        OAK_WOOD => (115, 82, 45),
        OAK_LEAF => (56, 152, 44),
        JUNGLE_LEAF => (38, 180, 55),
        ACACIA_WOOD => (168, 90, 38),
        ACACIA_LEAF => (128, 148, 60),
        CLOUD => (240, 245, 255),
        SNOW_BLOCK => (235, 248, 255),
        SPRUCE_LEAF => (38, 104, 50),
        DIRT => (130, 94, 50),
        GRAVEL => (132, 112, 98),
        BEDROCK => (18, 16, 16),
        _ => (0, 0, 0),
    }
}
#[inline]
fn block_char(hit: u8) -> char {
    match hit {
        WATER_S | WATER_D => '≈',
        OAK_LEAF | SPRUCE_LEAF | JUNGLE_LEAF | ACACIA_LEAF => '▒',
        _ => '█',
    }
}
#[inline]
fn pos_hash(x: i32, y: i32) -> u32 {
    (x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663)).wrapping_mul(83_492_791) as u32
}
fn surface_block(depth: i32, biome: u8, terrain_h: f32) -> u8 {
    match biome {
        BIOME_MOUNTAIN => {
            if depth == 0 {
                if terrain_h > 28.0 {
                    SNOW_BLOCK
                } else if terrain_h > 22.0 {
                    STONE
                } else {
                    GRASS
                }
            } else {
                STONE
            }
        }
        BIOME_DESERT | BIOME_OCEAN => {
            if depth <= 4 {
                SAND
            } else {
                STONE
            }
        }
        BIOME_SNOW => {
            if depth == 0 {
                SNOW_BLOCK
            } else if depth <= 3 {
                GRAVEL
            } else {
                STONE
            }
        }
        BIOME_SAVANNA => {
            if depth == 0 {
                SAVANNA_GRASS
            } else if depth <= 3 {
                DIRT
            } else {
                STONE
            }
        }
        BIOME_SWAMP => {
            if depth == 0 {
                SWAMP_GRASS
            } else if depth <= 3 {
                DIRT
            } else {
                STONE
            }
        }
        _ => {
            if depth == 0 {
                GRASS
            } else if depth <= 3 {
                DIRT
            } else if depth <= 14 {
                STONE
            } else {
                BEDROCK
            }
        }
    }
}
fn max_leaf_d2(rel_z: i32, biome: u8) -> i32 {
    match biome {
        BIOME_SNOW => match rel_z {
            i32::MIN..=-3 => -1,
            -2..=-1 => 2,
            0 => 1,
            1 => 0,
            _ => -1,
        },
        BIOME_SAVANNA => match rel_z {
            i32::MIN..=-2 => -1,
            -1 => 16,
            0 => 12,
            1 => 4,
            _ => -1,
        },
        BIOME_JUNGLE => match rel_z {
            i32::MIN..=-5 => -1,
            -4..=-1 => 8,
            0 => 4,
            1..=2 => 2,
            _ => -1,
        },
        _ => match rel_z {
            i32::MIN..=-4 => -1,
            -3..=-1 => 12,
            0 => 8,
            1 => 4,
            2 => 1,
            _ => -1,
        },
    }
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
    pitch_override: Option<f64>,
    speed: f64,
    height_offset: f64,
    perlin_elev: Perlin,
    perlin_elev2: Perlin,
    perlin_biome: Perlin,
    cache: Vec<CacheCell>,
    cache_min_x: i32,
    cache_min_y: i32,
}
impl MinecraftVisualizer {
    pub fn new(width: u16, height: u16, _palette: ThemePalette, _charset: CharSet) -> Self {
        Self {
            width,
            height,
            time: 0.0,
            cam_x: 0.0,
            cam_y: 0.0,
            cam_z: 40.0,
            yaw: 0.0,
            pitch: -0.35,
            pitch_override: None,
            speed: 2.0,
            height_offset: 16.0,
            perlin_elev: Perlin::new(rand::random::<u32>()),
            perlin_elev2: Perlin::new(rand::random::<u32>()),
            perlin_biome: Perlin::new(rand::random::<u32>()),
            cache: vec![CacheCell::default(); (CACHE_W * CACHE_W) as usize],
            cache_min_x: i32::MAX,
            cache_min_y: i32::MAX,
        }
    }
    #[inline]
    fn terrain_height(&self, x: f64, y: f64) -> f32 {
        let large = self.perlin_elev.get([x * 0.010, y * 0.010]) * 9.0;
        let med = self.perlin_elev2.get([x * 0.028, y * 0.028]) * 3.0;
        let fine = self.perlin_elev.get([x * 0.065, y * 0.065]) * 0.7;
        let bv = self.perlin_biome.get([x * 0.007, y * 0.007]);
        let mountain = (bv - 0.32).max(0.0) * 22.0;
        (14.0 + large + med + fine + mountain) as f32
    }
    #[inline]
    fn biome_id(&self, x: f64, y: f64) -> u8 {
        let v = self.perlin_biome.get([x * 0.007, y * 0.007]);
        let v2 = self
            .perlin_biome
            .get([x * 0.014 + 300.0, y * 0.014 + 300.0]);
        if v < -0.45 {
            BIOME_OCEAN
        } else if v < -0.20 {
            if v2 > 0.15 { BIOME_SWAMP } else { BIOME_PLAINS }
        } else if v < 0.05 {
            if v2 > 0.20 {
                BIOME_FOREST
            } else {
                BIOME_PLAINS
            }
        } else if v < 0.32 {
            if v2 > 0.15 {
                BIOME_JUNGLE
            } else {
                BIOME_SAVANNA
            }
        } else if v < 0.55 {
            if v2 < -0.10 {
                BIOME_DESERT
            } else {
                BIOME_MOUNTAIN
            }
        } else {
            BIOME_SNOW
        }
    }
    fn rebuild_cache(&mut self) {
        let min_x = self.cam_x.floor() as i32 - CACHE_W / 2;
        let min_y = self.cam_y.floor() as i32 - CACHE_W / 2;
        self.cache_min_x = min_x;
        self.cache_min_y = min_y;
        let mut tree_canopy = vec![0.0f32; (CACHE_W * CACHE_W) as usize];
        for cy in 0..CACHE_W {
            for cx in 0..CACHE_W {
                let wx = (min_x + cx) as f64;
                let wy = (min_y + cy) as f64;
                let biome = self.biome_id(wx, wy);
                let h = self.terrain_height(wx, wy);
                let idx = (cy * CACHE_W + cx) as usize;
                self.cache[idx] = CacheCell {
                    terrain_h: h,
                    canopy_top: h,
                    trunk_top: 0.0,
                    biome,
                };
                if h >= WATER_LEVEL as f32 {
                    let grid: i32 = match biome {
                        BIOME_JUNGLE => 8,
                        BIOME_FOREST => 11,
                        BIOME_SWAMP => 14,
                        BIOME_SAVANNA => 14,
                        _ => 0,
                    };
                    if grid > 0 {
                        let wx_i = min_x + cx;
                        let wy_i = min_y + cy;
                        let gx = wx_i.div_euclid(grid);
                        let gy = wy_i.div_euclid(grid);
                        let gh = pos_hash(gx, gy);
                        let margin = 2i32;
                        let jrange = (grid - margin * 2).max(1) as u32;
                        let jx = margin + (gh % jrange) as i32;
                        let jy = margin + ((gh >> 8) % jrange) as i32;
                        if gx * grid + jx == wx_i && gy * grid + jy == wy_i {
                            let hash = pos_hash(wx_i, wy_i);
                            let trunk_h = match biome {
                                BIOME_JUNGLE => 8.0 + (hash % 5) as f32,
                                BIOME_SAVANNA => 2.0 + (hash % 2) as f32,
                                _ => 4.0 + (hash % 3) as f32,
                            };
                            let trunk_top = h + trunk_h;
                            let leaf_below = if biome == BIOME_JUNGLE { 4.0 } else { 3.0 };
                            let canopy = trunk_top + 3.0;
                            let guard = trunk_top + leaf_below;
                            self.cache[idx].trunk_top = trunk_top;
                            self.cache[idx].canopy_top = guard.max(canopy);
                            tree_canopy[idx] = guard.max(canopy);
                        }
                    }
                }
            }
        }
        for cy in 0..CACHE_W {
            for cx in 0..CACHE_W {
                let src_canopy = tree_canopy[(cy * CACHE_W + cx) as usize];
                if src_canopy <= 0.0 {
                    continue;
                }
                let src_h = self.cache[(cy * CACHE_W + cx) as usize].terrain_h;
                for dy in -4i32..=4 {
                    for dx in -4i32..=4 {
                        if dx.abs() == 4 && dy.abs() == 4 {
                            continue;
                        }
                        let nx = cx + dx;
                        let ny = cy + dy;
                        if nx >= 0 && nx < CACHE_W && ny >= 0 && ny < CACHE_W {
                            let dst = (ny * CACHE_W + nx) as usize;
                            if self.cache[dst].terrain_h >= src_h - 5.0
                                && src_canopy > self.cache[dst].canopy_top
                            {
                                self.cache[dst].canopy_top = src_canopy;
                            }
                        }
                    }
                }
            }
        }
    }
    #[inline]
    fn get_cell(&self, x: i32, y: i32) -> CacheCell {
        let cx = (x - self.cache_min_x).clamp(0, CACHE_W - 1);
        let cy = (y - self.cache_min_y).clamp(0, CACHE_W - 1);
        self.cache[(cy * CACHE_W + cx) as usize]
    }
    fn tree_block_at(&self, x: i32, y: i32, z: i32) -> u8 {
        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let nc = self.get_cell(x + dx, y + dy);
                if nc.trunk_top <= 0.0 {
                    continue;
                }
                if z as f32 <= nc.terrain_h {
                    continue;
                }
                let d2 = dx * dx + dy * dy;
                if dx == 0 && dy == 0 && z as f32 <= nc.trunk_top {
                    return match nc.biome {
                        BIOME_SAVANNA => ACACIA_WOOD,
                        _ => OAK_WOOD,
                    };
                }
                let rel_z = z - nc.trunk_top as i32;
                let md2 = max_leaf_d2(rel_z, nc.biome);
                if md2 >= 0 && d2 <= md2 {
                    return match nc.biome {
                        BIOME_SNOW | BIOME_MOUNTAIN => SPRUCE_LEAF,
                        BIOME_JUNGLE => JUNGLE_LEAF,
                        BIOME_SAVANNA => ACACIA_LEAF,
                        _ => OAK_LEAF,
                    };
                }
            }
        }
        AIR
    }
}
impl Visualizer for MinecraftVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = delta_time * self.speed;
        self.time += delta_time;
        let move_speed = 3.5;
        self.cam_x += dt * self.yaw.sin() * move_speed;
        self.cam_y += dt * self.yaw.cos() * move_speed;
        self.yaw += delta_time * 0.015;
        let ground = if self.cache_min_x != i32::MAX {
            self.get_cell(self.cam_x.floor() as i32, self.cam_y.floor() as i32)
                .terrain_h as f64
        } else {
            14.0
        }
        .max(WATER_LEVEL as f64);
        let target_z = ground + self.height_offset;
        let spring = (delta_time * 2.5).min(0.30);
        self.cam_z += (target_z - self.cam_z) * spring;
        if let Some(target_pitch) = self.pitch_override {
            self.pitch += (target_pitch - self.pitch) * (delta_time * 4.0).min(0.5);
        } else {
            self.pitch = -0.35 + (self.time * 0.18).sin() * 0.025;
        }
    }
    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
        }
        buffer.clear();
        let cam_ix = self.cam_x.floor() as i32;
        let cam_iy = self.cam_y.floor() as i32;
        if self.cache_min_x == i32::MAX
            || (cam_ix - (self.cache_min_x + CACHE_W / 2)).abs() > CACHE_W / 5
            || (cam_iy - (self.cache_min_y + CACHE_W / 2)).abs() > CACHE_W / 5
        {
            self.rebuild_cache();
        }
        let day_cycle = (self.time * 0.005) % (std::f64::consts::PI * 2.0);
        let sun_h = day_cycle.sin();
        let (sky_r, sky_g, sky_b) = if sun_h > 0.25 {
            (105.0f64, 175.0, 235.0)
        } else if sun_h > -0.25 {
            let t = (sun_h + 0.25) / 0.5;
            let peak = 1.0 - ((t - 0.5).abs() * 2.0);
            (
                (22.0 * (1.0 - t) + 105.0 * t + peak * 135.0).clamp(0.0, 255.0),
                (12.0 * (1.0 - t) + 175.0 * t + peak * 28.0).clamp(0.0, 255.0),
                (22.0 * (1.0 - t) + 235.0 * t - peak * 65.0).clamp(0.0, 255.0),
            )
        } else {
            (8.0, 10.0, 28.0)
        };
        let global_dim = (0.18 + (sun_h + 1.0) * 0.41).clamp(0.18, 1.0);
        let light_angle = day_cycle;
        let dir_x = self.yaw.sin() * self.pitch.cos();
        let dir_y = self.yaw.cos() * self.pitch.cos();
        let dir_z = self.pitch.sin();
        let right_x = self.yaw.cos();
        let right_y = -self.yaw.sin();
        let up_x = right_y * dir_z;
        let up_y = -right_x * dir_z;
        let up_z = right_x * dir_y - right_y * dir_x;
        let fov_h = 0.82;
        let fov_v = 0.44;
        let max_steps = 400usize;
        for sy in 0..self.height {
            for sx in 0..self.width {
                let ndc_x = (2.0 * sx as f64 / self.width as f64 - 1.0) * fov_h;
                let ndc_y = (1.0 - 2.0 * sy as f64 / self.height as f64) * fov_v;
                let mut rx = dir_x + right_x * ndc_x + up_x * ndc_y;
                let mut ry = dir_y + right_y * ndc_x + up_y * ndc_y;
                let mut rz = dir_z + up_z * ndc_y;
                let rlen = (rx * rx + ry * ry + rz * rz).sqrt();
                rx /= rlen;
                ry /= rlen;
                rz /= rlen;
                let mut map_x = self.cam_x.floor() as i32;
                let mut map_y = self.cam_y.floor() as i32;
                let mut map_z = self.cam_z.floor() as i32;
                let ddx = if rx == 0.0 { 1e30 } else { (1.0 / rx).abs() };
                let ddy = if ry == 0.0 { 1e30 } else { (1.0 / ry).abs() };
                let ddz = if rz == 0.0 { 1e30 } else { (1.0 / rz).abs() };
                let sx_step = if rx < 0.0 { -1i32 } else { 1 };
                let sy_step = if ry < 0.0 { -1i32 } else { 1 };
                let sz_step = if rz < 0.0 { -1i32 } else { 1 };
                let mut sdx = if rx < 0.0 {
                    (self.cam_x - map_x as f64) * ddx
                } else {
                    (map_x as f64 + 1.0 - self.cam_x) * ddx
                };
                let mut sdy = if ry < 0.0 {
                    (self.cam_y - map_y as f64) * ddy
                } else {
                    (map_y as f64 + 1.0 - self.cam_y) * ddy
                };
                let mut sdz = if rz < 0.0 {
                    (self.cam_z - map_z as f64) * ddz
                } else {
                    (map_z as f64 + 1.0 - self.cam_z) * ddz
                };
                let mut hit: u8 = AIR;
                let mut side: u8 = 2;
                'march: for _ in 0..max_steps {
                    if sdx < sdy {
                        if sdx < sdz {
                            sdx += ddx;
                            map_x += sx_step;
                            side = 0;
                        } else {
                            sdz += ddz;
                            map_z += sz_step;
                            side = 2;
                        }
                    } else if sdy < sdz {
                        sdy += ddy;
                        map_y += sy_step;
                        side = 1;
                    } else {
                        sdz += ddz;
                        map_z += sz_step;
                        side = 2;
                    }
                    if map_z >= 68 && map_z <= 70 {
                        let drift = (self.time * 1.5) as i32;
                        let cx = map_x.wrapping_sub(drift);
                        let gx = cx.div_euclid(12);
                        let gy = map_y.div_euclid(12);
                        if pos_hash(gx, gy) % 100 < 10 {
                            hit = CLOUD;
                            break 'march;
                        }
                    }
                    let cell = self.get_cell(map_x, map_y);
                    let depth = cell.terrain_h as i32 - map_z;
                    if depth >= 0 {
                        let exposed = match side {
                            0 => (self.get_cell(map_x - sx_step, map_y).terrain_h as i32) < map_z,
                            1 => (self.get_cell(map_x, map_y - sy_step).terrain_h as i32) < map_z,
                            _ => true,
                        };
                        if exposed {
                            let vis_depth = if side == 2 { depth } else { depth.min(3) };
                            hit = surface_block(vis_depth, cell.biome, cell.terrain_h);
                        }
                        break 'march;
                    }
                    if map_z as f32 > cell.terrain_h && map_z as f32 <= cell.canopy_top {
                        let tree_hit = self.tree_block_at(map_x, map_y, map_z);
                        if tree_hit != AIR {
                            hit = tree_hit;
                            break 'march;
                        }
                    }
                    if map_z <= WATER_LEVEL && (cell.terrain_h as i32) < WATER_LEVEL {
                        hit = if map_z == WATER_LEVEL {
                            WATER_S
                        } else {
                            WATER_D
                        };
                        side = 2;
                        break 'march;
                    }
                }
                if hit == AIR {
                    let (ch, fg) = if sun_h < -0.2 {
                        let sh = (sx as u32)
                            .wrapping_mul(9_123)
                            .wrapping_add((sy as u32).wrapping_mul(5_829))
                            .wrapping_mul(1_239);
                        if sh % 120 < 2 {
                            (
                                '.',
                                Color::Rgb {
                                    r: 210,
                                    g: 215,
                                    b: 255,
                                },
                            )
                        } else {
                            let mx = (self.width as f64 * 0.78) as u16;
                            let my = (self.height as f64 * 0.14) as u16;
                            if sx.abs_diff(mx) <= 1 && sy.abs_diff(my) <= 1 {
                                (
                                    '○',
                                    Color::Rgb {
                                        r: 240,
                                        g: 240,
                                        b: 220,
                                    },
                                )
                            } else {
                                (' ', Color::Reset)
                            }
                        }
                    } else {
                        (' ', Color::Reset)
                    };
                    let sky_t = sy as f64 / self.height as f64;
                    let dim = 0.82 + sky_t * 0.18;
                    buffer.set(
                        sx,
                        sy,
                        ch,
                        fg,
                        Color::Rgb {
                            r: (sky_r * dim).min(255.0) as u8,
                            g: (sky_g * dim).min(255.0) as u8,
                            b: (sky_b * dim).min(255.0) as u8,
                        },
                    );
                    continue;
                }
                let perp = match side {
                    0 => sdx - ddx,
                    1 => sdy - ddy,
                    _ => sdz - ddz,
                };
                let ch = if hit == WATER_S {
                    let wave = (map_x as f64 * 1.1 + map_y as f64 * 0.7 + self.time * 3.0).sin();
                    if wave > 0.3 { '▓' } else { '▒' }
                } else if hit == WATER_D {
                    '█'
                } else {
                    block_char(hit)
                };
                let (mut r, mut g, mut b) = block_color(hit);
                if hit == WATER_S || hit == WATER_D {
                    let shimmer = if hit == WATER_S {
                        let w = (map_x as f64 * 1.1 + map_y as f64 * 0.7 + self.time * 3.0).sin();
                        0.88 + w * 0.12
                    } else {
                        0.72
                    };
                    let dim = (shimmer * global_dim) as f32;
                    r = (r as f32 * dim) as u8;
                    g = (g as f32 * dim) as u8;
                    b = (b as f32 * dim) as u8;
                    if hit == WATER_S {
                        r = (r as f64 * 0.80 + sky_r * 0.20) as u8;
                        g = (g as f64 * 0.80 + sky_g * 0.20) as u8;
                        b = (b as f64 * 0.80 + sky_b * 0.20) as u8;
                    }
                    let fog = (-perp * 0.004).exp().clamp(0.0, 1.0);
                    let fr = (r as f64 * fog + sky_r * (1.0 - fog)) as u8;
                    let fg_c = (g as f64 * fog + sky_g * (1.0 - fog)) as u8;
                    let fb = (b as f64 * fog + sky_b * (1.0 - fog)) as u8;
                    buffer.set(
                        sx,
                        sy,
                        ch,
                        Color::Rgb {
                            r: fr,
                            g: fg_c,
                            b: fb,
                        },
                        Color::Reset,
                    );
                    continue;
                }
                let top_var = if side == 2 && hit != CLOUD {
                    let v = pos_hash(map_x, map_y) % 18;
                    1.0 - v as f64 * 0.005
                } else {
                    1.0
                };
                let face_dim = if hit == CLOUD {
                    0.95
                } else {
                    match side {
                        0 => 0.65,
                        1 => 0.80,
                        _ => 1.00,
                    }
                };
                let dim = (face_dim * global_dim * top_var) as f32;
                r = (r as f32 * dim) as u8;
                g = (g as f32 * dim) as u8;
                b = (b as f32 * dim) as u8;
                let fog = (-perp * 0.004).exp().clamp(0.0, 1.0);
                let fr = (r as f64 * fog + sky_r * (1.0 - fog)) as u8;
                let fg_c = (g as f64 * fog + sky_g * (1.0 - fog)) as u8;
                let fb = (b as f64 * fog + sky_b * (1.0 - fog)) as u8;
                buffer.set(
                    sx,
                    sy,
                    ch,
                    Color::Rgb {
                        r: fr,
                        g: fg_c,
                        b: fb,
                    },
                    Color::Reset,
                );
            }
        }
    }
    fn set_palette(&mut self, _palette: ThemePalette) {}
    fn set_charset(&mut self, _charset: CharSet) {}
    fn on_scroll(&mut self, delta: i32) {
        self.speed = (self.speed + delta as f64).clamp(0.5, 60.0);
    }
    fn on_scroll_ext(&mut self, delta: i32, is_ctrl: bool) {
        if is_ctrl {
            let current = self.pitch_override.unwrap_or(self.pitch);
            let next = (current - delta as f64 * 0.12).clamp(-std::f64::consts::FRAC_PI_2, -0.10);
            self.pitch_override = Some(next);
        } else {
            self.on_scroll(delta);
        }
    }
    fn on_key(
        &mut self,
        code: crossterm::event::KeyCode,
        _mods: crossterm::event::KeyModifiers,
    ) -> bool {
        match code {
            crossterm::event::KeyCode::Up => {
                self.height_offset = (self.height_offset + 2.0).min(100.0);
                true
            }
            crossterm::event::KeyCode::Down => {
                self.height_offset = (self.height_offset - 2.0).max(3.0);
                true
            }
            _ => false,
        }
    }
}
