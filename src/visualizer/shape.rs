use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::{ThemePalette, interpolate_gradient};
use crate::visualizer::Visualizer;
use std::f64::consts::PI;

#[derive(Clone, Copy)]
struct Point3D {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Clone, Copy)]
struct Point4D {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

pub struct ShapeVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    angle_x: f64,
    angle_y: f64,
    angle_z: f64,
    angle_w: f64,
    zoom: f64,
    shape_index: usize,
    cube_points: Vec<Point3D>,
    cube_edges: Vec<(usize, usize)>,
    tesseract_points: Vec<Point4D>,
    tesseract_edges: Vec<(usize, usize)>,
}

impl ShapeVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut cube_points = Vec::new();
        for x in [-1.0, 1.0].iter() {
            for y in [-1.0, 1.0].iter() {
                for z in [-1.0, 1.0].iter() {
                    cube_points.push(Point3D {
                        x: *x,
                        y: *y,
                        z: *z,
                    });
                }
            }
        }
        let cube_edges = vec![
            (0, 1),
            (1, 3),
            (3, 2),
            (2, 0),
            (4, 5),
            (5, 7),
            (7, 6),
            (6, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        let mut tesseract_points = Vec::new();
        for x in [-1.0, 1.0].iter() {
            for y in [-1.0, 1.0].iter() {
                for z in [-1.0, 1.0].iter() {
                    for w in [-1.0, 1.0].iter() {
                        tesseract_points.push(Point4D {
                            x: *x,
                            y: *y,
                            z: *z,
                            w: *w,
                        });
                    }
                }
            }
        }
        let mut tesseract_edges = Vec::new();
        for i in 0..16 {
            for j in (i + 1)..16 {
                let mut diffs = 0;
                if tesseract_points[i].x != tesseract_points[j].x {
                    diffs += 1;
                }
                if tesseract_points[i].y != tesseract_points[j].y {
                    diffs += 1;
                }
                if tesseract_points[i].z != tesseract_points[j].z {
                    diffs += 1;
                }
                if tesseract_points[i].w != tesseract_points[j].w {
                    diffs += 1;
                }
                if diffs == 1 {
                    tesseract_edges.push((i, j));
                }
            }
        }

        Self {
            width,
            height,
            palette,
            charset,
            angle_x: 0.0,
            angle_y: 0.0,
            angle_z: 0.0,
            angle_w: 0.0,
            zoom: 3.5,
            shape_index: 0,
            cube_points,
            cube_edges,
            tesseract_points,
            tesseract_edges,
        }
    }

    fn draw_line(
        &self,
        buffer: &mut ScreenBuffer,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: crossterm::style::Color,
    ) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;

        let char_idx = if dx > -dy { 0 } else { 1 };
        let c = if self.charset.chars.len() > char_idx {
            self.charset.chars[char_idx]
        } else {
            '*'
        };

        loop {
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                buffer.set(x as u16, y as u16, c, color, crossterm::style::Color::Reset);
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn project(&self, p: Point3D) -> (i32, i32) {
        let y1 = p.y * self.angle_x.cos() - p.z * self.angle_x.sin();
        let z1 = p.y * self.angle_x.sin() + p.z * self.angle_x.cos();
        let x1 = p.x;

        let x2 = x1 * self.angle_y.cos() + z1 * self.angle_y.sin();
        let _z2 = -x1 * self.angle_y.sin() + z1 * self.angle_y.cos();
        let y2 = y1;

        let x3 = x2 * self.angle_z.cos() - y2 * self.angle_z.sin();
        let y3 = x2 * self.angle_z.sin() + y2 * self.angle_z.cos();

        let f = self.zoom * (self.height as f64 / 10.0);

        let px = (self.width as f64 / 2.0 + x3 * f * 2.0).round() as i32;
        let py = (self.height as f64 / 2.0 + y3 * f).round() as i32;

        (px, py)
    }

    fn project_4d(&self, p: Point4D) -> Point3D {
        let x1 = p.x * self.angle_w.cos() - p.w * self.angle_w.sin();
        let w1 = p.x * self.angle_w.sin() + p.w * self.angle_w.cos();

        let distance = 3.0;
        let w_factor = 1.0 / (distance - w1);

        Point3D {
            x: x1 * w_factor,
            y: p.y * w_factor,
            z: p.z * w_factor,
        }
    }
}

impl Visualizer for ShapeVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.angle_x += 0.5 * delta_time;
        self.angle_y += 0.7 * delta_time;
        self.angle_z += 0.3 * delta_time;
        self.angle_w += 0.9 * delta_time;
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

        let color = interpolate_gradient(&self.palette, 0.7);

        match self.shape_index {
            0 => {
                for &(i, j) in &self.cube_edges {
                    let (x0, y0) = self.project(self.cube_points[i]);
                    let (x1, y1) = self.project(self.cube_points[j]);
                    self.draw_line(buffer, x0, y0, x1, y1, color);
                }
            }
            1 => {
                let radius = 1.5;
                let resolution = 16;
                let mut points = Vec::new();

                for lat in 0..=resolution {
                    let phi = PI * (lat as f64) / (resolution as f64);
                    let mut row = Vec::new();
                    for lon in 0..resolution {
                        let theta = 2.0 * PI * (lon as f64) / (resolution as f64);
                        let x = radius * phi.sin() * theta.cos();
                        let y = radius * phi.sin() * theta.sin();
                        let z = radius * phi.cos();
                        row.push(Point3D { x, y, z });
                    }
                    points.push(row);
                }

                for lat in 0..resolution {
                    for lon in 0..resolution {
                        let p1 = points[lat][lon];
                        let p2 = points[lat + 1][lon];
                        let p3 = points[lat][(lon + 1) % resolution];

                        let (x0, y0) = self.project(p1);
                        let (x1, y1) = self.project(p2);
                        let (x2, y2) = self.project(p3);

                        self.draw_line(buffer, x0, y0, x1, y1, color);
                        self.draw_line(buffer, x0, y0, x2, y2, color);
                    }
                }
            }
            2 => {
                let big_r = 1.2;
                let small_r = 0.5;
                let res_ring = 24;
                let res_tube = 12;
                let mut points = Vec::new();

                for i in 0..res_ring {
                    let theta = 2.0 * PI * (i as f64) / (res_ring as f64);
                    let mut ring = Vec::new();
                    for j in 0..res_tube {
                        let phi = 2.0 * PI * (j as f64) / (res_tube as f64);
                        let x = (big_r + small_r * phi.cos()) * theta.cos();
                        let y = (big_r + small_r * phi.cos()) * theta.sin();
                        let z = small_r * phi.sin();
                        ring.push(Point3D { x, y, z });
                    }
                    points.push(ring);
                }

                for i in 0..res_ring {
                    for j in 0..res_tube {
                        let p1 = points[i][j];
                        let p2 = points[(i + 1) % res_ring][j];
                        let p3 = points[i][(j + 1) % res_tube];

                        let (x0, y0) = self.project(p1);
                        let (x1, y1) = self.project(p2);
                        let (x2, y2) = self.project(p3);

                        self.draw_line(buffer, x0, y0, x1, y1, color);
                        self.draw_line(buffer, x0, y0, x2, y2, color);
                    }
                }
            }
            3 => {
                let mut projected_3d = Vec::new();
                for p in &self.tesseract_points {
                    projected_3d.push(self.project_4d(*p));
                }

                for &(i, j) in &self.tesseract_edges {
                    let (x0, y0) = self.project(projected_3d[i]);
                    let (x1, y1) = self.project(projected_3d[j]);
                    self.draw_line(buffer, x0, y0, x1, y1, color);
                }
            }
            _ => {}
        }
    }

    fn set_palette(&mut self, palette: ThemePalette) {
        self.palette = palette;
    }
    fn set_charset(&mut self, charset: CharSet) {
        self.charset = charset;
    }

    fn on_scroll_ext(&mut self, delta: i32, is_ctrl: bool) {
        if is_ctrl {
            self.zoom += (delta as f64) * 0.5;
            self.zoom = self.zoom.clamp(0.5, 20.0);
        } else {
            if delta > 0 {
                self.shape_index = (self.shape_index + 1) % 4;
            } else {
                self.shape_index = (self.shape_index + 3) % 4;
            }
        }
    }
}
