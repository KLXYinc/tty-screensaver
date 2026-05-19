use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemePalette;
use crate::visualizer::Visualizer;
use crossterm::style::Color;
use rand::Rng;

struct Fish {
    x: f64,
    y: f64,
    speed: f64,
    direction: f64,
    kind: usize,
    color: Color,
}

struct Bubble {
    x: f64,
    y: f64,
    speed: f64,
    offset: f64,
}

const FISH_SPRITES_RIGHT: [&str; 4] = ["><(((('>", "><>", "|\\ \n| >o \n|/ ", "  /\\ \n><(o)>"];

const FISH_SPRITES_LEFT: [&str; 4] = ["<'))))><", "<><", " /|\n o< |\n \\|", "  /\\ \n<(o)><"];

pub struct AquariumVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    time: f64,
    fishes: Vec<Fish>,
    bubbles: Vec<Bubble>,
    bg_enabled: bool,
}

impl AquariumVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        let mut vis = Self {
            width,
            height,
            palette,
            charset,
            time: 0.0,
            fishes: Vec::new(),
            bubbles: Vec::new(),
            bg_enabled: false,
        };
        vis.init_scene();
        vis
    }

    fn init_scene(&mut self) {
        let mut rng = rand::thread_rng();

        self.fishes.clear();
        for _ in 0..10 {
            self.spawn_fish();
            let last = self.fishes.last_mut().unwrap();
            last.x = rng.gen_range(0.0..(self.width as f64));
        }

        self.bubbles.clear();
        for _ in 0..20 {
            self.spawn_bubble();
            let last = self.bubbles.last_mut().unwrap();
            last.y = rng.gen_range(0.0..(self.height as f64));
        }
    }

    fn spawn_fish(&mut self) {
        let mut rng = rand::thread_rng();
        let dir = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        let x = if dir > 0.0 {
            -10.0
        } else {
            self.width as f64 + 10.0
        };
        let y = rng.gen_range(2.0..(self.height as f64 - 5.0));
        let speed = rng.gen_range(5.0..15.0);
        let kind = if rng.gen_bool(0.7) { 1 } else { 0 };

        let colors = [
            self.palette.colors.get(0).copied().unwrap_or(Color::White),
            self.palette.colors.get(1).copied().unwrap_or(Color::White),
            self.palette.colors.get(2).copied().unwrap_or(Color::White),
            Color::Rgb {
                r: 255,
                g: 100,
                b: 0,
            },
        ];
        let color = colors[rng.gen_range(0..colors.len())];

        self.fishes.push(Fish {
            x,
            y,
            speed,
            direction: dir,
            kind,
            color,
        });
    }

    fn spawn_bubble(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0.0..(self.width as f64));
        let y = self.height as f64 + 2.0;
        let speed = rng.gen_range(2.0..6.0);
        let offset = rng.gen_range(0.0..10.0);
        self.bubbles.push(Bubble {
            x,
            y,
            speed,
            offset,
        });
    }
}

impl Visualizer for AquariumVisualizer {
    fn update(&mut self, delta_time: f64) {
        self.time += delta_time;

        let mut dead_fishes = Vec::new();
        for (i, fish) in self.fishes.iter_mut().enumerate() {
            fish.x += fish.speed * fish.direction * delta_time;

            fish.y += (self.time * 2.0 + fish.x).sin() * 0.5 * delta_time;

            if (fish.direction > 0.0 && fish.x > self.width as f64 + 10.0)
                || (fish.direction < 0.0 && fish.x < -10.0)
            {
                dead_fishes.push(i);
            }
        }

        for i in dead_fishes.iter().rev() {
            self.fishes.remove(*i);
            self.spawn_fish();
        }

        let mut dead_bubbles = Vec::new();
        for (i, bubble) in self.bubbles.iter_mut().enumerate() {
            bubble.y -= bubble.speed * delta_time;
            if bubble.y < -2.0 {
                dead_bubbles.push(i);
            }
        }

        for i in dead_bubbles.iter().rev() {
            self.bubbles.remove(*i);
            self.spawn_bubble();
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
            self.init_scene();
        }

        let water_bg = if self.bg_enabled {
            Color::Rgb { r: 0, g: 10, b: 30 }
        } else {
            Color::Reset
        };
        if self.bg_enabled {
            for y in 0..self.height {
                for x in 0..self.width {
                    buffer.set(x, y, ' ', Color::Reset, water_bg);
                }
            }
        } else {
            buffer.clear();
        }

        let seaweed_color = Color::Rgb {
            r: 20,
            g: 150,
            b: 50,
        };
        for x in 0..self.width {
            if x % 4 == 0 || x % 7 == 0 {
                let height = 3 + (x % 5);
                for i in 0..height {
                    let y = self.height as i32 - 1 - i as i32;
                    let wave =
                        ((x as f64 * 0.2 + self.time * 2.0 + i as f64 * 0.5).sin() * 1.5) as i32;
                    let draw_x = x as i32 + wave;
                    if draw_x >= 0 && draw_x < self.width as i32 && y >= 0 {
                        buffer.set(draw_x as u16, y as u16, '~', seaweed_color, water_bg);
                    }
                }
            }
        }

        let bubble_color = Color::Rgb {
            r: 150,
            g: 200,
            b: 255,
        };
        for bubble in &self.bubbles {
            let wave_x = bubble.x + (self.time * 3.0 + bubble.offset).sin() * 2.0;
            if wave_x >= 0.0
                && wave_x < self.width as f64
                && bubble.y >= 0.0
                && bubble.y < self.height as f64
            {
                let ch = if (bubble.y as i32) % 3 == 0 { 'o' } else { 'O' };
                buffer.set(wave_x as u16, bubble.y as u16, ch, bubble_color, water_bg);
            }
        }

        for fish in &self.fishes {
            let sprite = if fish.direction > 0.0 {
                FISH_SPRITES_RIGHT[fish.kind]
            } else {
                FISH_SPRITES_LEFT[fish.kind]
            };

            for (dx, ch) in sprite.chars().enumerate() {
                let draw_x = fish.x as i32 + dx as i32;
                let draw_y = fish.y as i32;
                if draw_x >= 0
                    && draw_x < self.width as i32
                    && draw_y >= 0
                    && draw_y < self.height as i32
                {
                    if ch != ' ' {
                        buffer.set(draw_x as u16, draw_y as u16, ch, fish.color, water_bg);
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

    fn on_scroll(&mut self, _delta: i32) {
        self.bg_enabled = !self.bg_enabled;
    }
}
