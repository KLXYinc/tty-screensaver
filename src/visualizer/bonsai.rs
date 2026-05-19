use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemePalette;
use crate::visualizer::Visualizer;
use crossterm::style::Color;
use rand::Rng;

#[derive(Clone)]
struct Tip {
    x: f64,
    y: f64,
    angle: f64,
    life: i32,
    thickness: f64,
}

pub struct BonsaiVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,

    grid: Vec<(char, Color)>,
    tips: Vec<Tip>,
    accumulator: f64,
    timer: f64,
}

impl BonsaiVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            width,
            height,
            palette,
            charset,
            grid: vec![(' ', Color::Reset); (width as usize) * (height as usize)],
            tips: Vec::new(),
            accumulator: 0.0,
            timer: 0.0,
        };
        vis.restart();
        vis
    }

    fn restart(&mut self) {
        self.grid.fill((' ', Color::Reset));
        self.tips.clear();
        self.tips.push(Tip {
            x: self.width as f64 / 2.0,
            y: self.height as f64 - 2.0,
            angle: std::f64::consts::PI / 2.0,
            life: 25,
            thickness: 3.0,
        });
        self.timer = 0.0;
    }

    fn draw_circle(&mut self, cx: i32, cy: i32, radius: f64, ch: char, color: Color) {
        let r = radius.ceil() as i32;
        for y in -r..=r {
            for x in -r..=r {
                if (x * x + y * y) as f64 <= radius * radius {
                    let px = cx + x;
                    let py = cy + y;
                    if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                        self.grid[(py * self.width as i32 + px) as usize] = (ch, color);
                    }
                }
            }
        }
    }
}

impl Visualizer for BonsaiVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.accumulator += delta_time;
        self.timer += delta_time;

        if self.tips.is_empty() && self.timer > 5.0 {
            self.restart();
        }

        while self.accumulator >= 0.05 {
            self.accumulator -= 0.05;

            let mut new_tips = Vec::new();
            let mut rng = rand::thread_rng();

            let mut keep = Vec::new();

            let tips_to_process = std::mem::take(&mut self.tips);
            for mut tip in tips_to_process {
                let ch = if tip.thickness > 1.5 { '█' } else { '▓' };
                let trunk_color = Color::Rgb {
                    r: 139,
                    g: 69,
                    b: 19,
                };
                self.draw_circle(tip.x as i32, tip.y as i32, tip.thickness, ch, trunk_color);

                tip.x += tip.angle.cos() * 0.8;
                tip.y -= tip.angle.sin() * 0.8;
                tip.life -= 1;

                tip.angle += rng.gen_range(-0.2..0.2);

                if tip.life > 0 {
                    if rng.gen_bool(0.1) && tip.thickness > 0.5 {
                        let branch_angle = tip.angle
                            + rng.gen_range(0.3..0.8) * if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                        new_tips.push(Tip {
                            x: tip.x,
                            y: tip.y,
                            angle: branch_angle,
                            life: tip.life,
                            thickness: tip.thickness * 0.7,
                        });
                        tip.thickness *= 0.8;
                    }
                    keep.push(tip);
                } else {
                    let leaf_color = if rng.gen_bool(0.5) {
                        Color::Rgb {
                            r: 34,
                            g: 139,
                            b: 34,
                        }
                    } else {
                        Color::Rgb {
                            r: 107,
                            g: 142,
                            b: 35,
                        }
                    };

                    let leaf_ch = if rng.gen_bool(0.5) { '&' } else { '*' };
                    for _ in 0..15 {
                        let lx = tip.x as i32 + rng.gen_range(-3..=3);
                        let ly = tip.y as i32 + rng.gen_range(-2..=2);
                        if lx >= 0 && lx < self.width as i32 && ly >= 0 && ly < self.height as i32 {
                            self.grid[(ly * self.width as i32 + lx) as usize] =
                                (leaf_ch, leaf_color);
                        }
                    }
                }
            }

            self.tips.extend(keep);
            self.tips.extend(new_tips);

            if self.tips.is_empty() && self.timer <= 5.0 {
                self.timer = 0.0;
            }
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.grid = vec![(' ', Color::Reset); (self.width as usize) * (self.height as usize)];
            self.restart();
        }

        buffer.clear();

        for y in 0..self.height {
            for x in 0..self.width {
                let (ch, color) = self.grid[(y * self.width + x) as usize];
                if ch != ' ' {
                    buffer.set(x, y, ch, color, crossterm::style::Color::Reset);
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
}
