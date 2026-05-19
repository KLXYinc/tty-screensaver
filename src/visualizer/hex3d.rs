use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::utils::math3d::Vec3;
use crate::visualizer::Visualizer;
use std::f64::consts::PI;
struct Hexagon {
    q: i32,
    r: i32,
    world_x: f64,
    world_y: f64,
    world_z: f64,
    dist_to_cam: f64,
}
pub struct Hex3DVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    time: f64,
    cam_x: f64,
    cam_y: f64,
    cam_z: f64,
    pitch: f64,
    yaw: f64,
    roll: f64,
    speed: f64,
}
impl Hex3DVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        Self {
            width,
            height,
            palette,
            charset,
            time: 0.0,
            cam_x: 0.0,
            cam_y: 0.0,
            cam_z: 10.0,
            pitch: -0.4,
            yaw: 0.0,
            roll: 0.0,
            speed: 5.0,
        }
    }
    fn draw_line(
        &self,
        buffer: &mut ScreenBuffer,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        c: char,
        color: crossterm::style::Color,
    ) {
        buffer.draw_line(x0, y0, x1, y1, c, color, crossterm::style::Color::Reset);
    }
    fn project(&self, p: Vec3) -> Option<(i32, i32)> {
        let t = Vec3::new(p.x - self.cam_x, p.y - self.cam_y, p.z - self.cam_z);
        let rotated = t
            .rotate_z(self.yaw)
            .rotate_x(-self.pitch)
            .rotate_y(-self.roll);
        if rotated.y < 0.1 {
            return None;
        }
        let fov = self.width as f64 * 0.4;
        let screen_x =
            (self.width as f64 / 2.0 + (rotated.x / rotated.y) * fov * 2.0).round() as i32;
        let screen_y = (self.height as f64 / 2.0 - (rotated.z / rotated.y) * fov).round() as i32;
        Some((screen_x, screen_y))
    }
}
impl Visualizer for Hex3DVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = delta_time * self.speed;
        self.time += dt;
        self.cam_y += dt * 8.0;
        self.cam_z = 10.0 + (self.time * 0.5).sin() * 2.0;
        self.pitch = -0.4 + (self.time * 0.3).sin() * 0.15;
        self.roll = (self.time * 0.4).cos() * 0.2;
        self.yaw = (self.time * 0.2).sin() * 0.3;
    }
    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
        }
        buffer.clear();
        if self.charset.chars.is_empty() {
            return;
        }
        let hex_size = 2.0;
        let view_distance = 12;
        let cam_q = (self.cam_x * (3.0_f64).sqrt() / 3.0 - self.cam_y / 3.0) / hex_size;
        let cam_r = (self.cam_y * 2.0 / 3.0) / hex_size;
        let q_center = cam_q.round() as i32;
        let r_center = cam_r.round() as i32;
        let mut hexes = Vec::new();
        for q in (q_center - view_distance)..=(q_center + view_distance) {
            for r in (r_center - view_distance)..=(r_center + view_distance) {
                let world_x = hex_size * 3.0_f64.sqrt() * (q as f64 + r as f64 / 2.0);
                let world_y = hex_size * 1.5 * r as f64;
                let dist_from_origin = (world_x.powi(2) + world_y.powi(2)).sqrt();
                let height = (world_x * 0.2).sin() * 3.0
                    + (world_y * 0.15).cos() * 2.0
                    + (dist_from_origin * 0.1).sin() * 2.0;
                let dx = world_x - self.cam_x;
                let dy = world_y - self.cam_y;
                let dz = height - self.cam_z;
                let dist_to_cam = (dx * dx + dy * dy + dz * dz).sqrt();
                let forward_x = -self.yaw.sin();
                let forward_y = self.yaw.cos();
                let dot = dx * forward_x + dy * forward_y;
                if dot > -10.0 {
                    hexes.push(Hexagon {
                        q,
                        r,
                        world_x,
                        world_y,
                        world_z: height,
                        dist_to_cam,
                    });
                }
            }
        }
        hexes.sort_by(|a, b| b.dist_to_cam.partial_cmp(&a.dist_to_cam).unwrap());
        let mut corners = Vec::new();
        for i in 0..6 {
            let angle = 2.0 * PI * (i as f64) / 6.0;
            corners.push((hex_size * angle.cos(), hex_size * angle.sin()));
        }
        for hex in &hexes {
            let normalized_height = ((hex.world_z + 7.0) / 14.0).clamp(0.0, 1.0);
            let color = interpolate_gradient(&self.palette, normalized_height as f32);
            let char_idx =
                (normalized_height * (self.charset.chars.len() as f64 - 1.0)).round() as usize;
            let c = self.charset.chars[char_idx.clamp(0, self.charset.chars.len() - 1)];
            let mut projected_corners = Vec::new();
            let mut all_valid = true;
            for offset in &corners {
                let p = Vec3::new(hex.world_x + offset.0, hex.world_y + offset.1, hex.world_z);
                if let Some(proj) = self.project(p) {
                    projected_corners.push(proj);
                } else {
                    all_valid = false;
                    break;
                }
            }
            if all_valid && projected_corners.len() == 6 {
                for i in 0..6 {
                    let p1 = projected_corners[i];
                    let p2 = projected_corners[(i + 1) % 6];
                    self.draw_line(buffer, p1.0, p1.1, p2.0, p2.1, c, color);
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
        self.speed += delta as f64 * 1.0;
        self.speed = self.speed.clamp(1.0, 50.0);
    }
}
