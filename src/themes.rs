use crossterm::style::Color;
#[derive(Clone, PartialEq, Eq)]
pub struct ThemePalette {
    pub colors: Vec<Color>,
    pub background: Color,
}
impl ThemePalette {
    pub fn new(background: Color, colors: Vec<Color>) -> Self {
        Self { colors, background }
    }
    pub fn primary(&self) -> Color {
        self.colors.get(0).copied().unwrap_or(Color::White)
    }
    pub fn secondary(&self) -> Color {
        self.colors.get(1).copied().unwrap_or(Color::White)
    }
    pub fn accent(&self) -> Color {
        self.colors.get(2).copied().unwrap_or(Color::White)
    }
}
#[derive(Clone)]
pub struct ThemeDef {
    pub name: &'static str,
    pub palette: ThemePalette,
}
pub fn interpolate_gradient(palette: &ThemePalette, ratio: f32) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);
    if palette.colors.is_empty() {
        return palette.background;
    }
    let segments = (palette.colors.len() - 1) as f32;
    if segments <= 0.0 {
        return palette.colors[0];
    }
    let scaled = ratio * segments;
    let idx = scaled.floor() as usize;
    let t = scaled - scaled.floor();
    if idx >= palette.colors.len() - 1 {
        return *palette.colors.last().unwrap();
    }
    let c1 = palette.colors[idx];
    let c2 = palette.colors[idx + 1];
    let (r1, g1, b1) = match c1 {
        Color::Rgb { r, g, b } => (r as f32, g as f32, b as f32),
        _ => (255.0, 255.0, 255.0),
    };
    let (r2, g2, b2) = match c2 {
        Color::Rgb { r, g, b } => (r as f32, g as f32, b as f32),
        _ => (255.0, 255.0, 255.0),
    };
    Color::Rgb {
        r: (r1 + (r2 - r1) * t) as u8,
        g: (g1 + (g2 - g1) * t) as u8,
        b: (b1 + (b2 - b1) * t) as u8,
    }
}
pub fn get_all_themes() -> Vec<ThemeDef> {
    vec![
        ThemeDef {
            name: "Default Matrix",
            palette: ThemePalette::new(
                Color::Rgb { r: 0, g: 0, b: 0 },
                vec![
                    Color::Rgb { r: 0, g: 20, b: 0 },
                    Color::Rgb { r: 0, g: 100, b: 0 },
                    Color::Rgb { r: 0, g: 255, b: 0 },
                    Color::Rgb {
                        r: 200,
                        g: 255,
                        b: 200,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Doom Fire",
            palette: ThemePalette::new(
                Color::Rgb { r: 0, g: 0, b: 0 },
                vec![
                    Color::Rgb { r: 20, g: 0, b: 0 },
                    Color::Rgb { r: 100, g: 0, b: 0 },
                    Color::Rgb { r: 255, g: 0, b: 0 },
                    Color::Rgb {
                        r: 255,
                        g: 128,
                        b: 0,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 0,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Rainbow",
            palette: ThemePalette::new(
                Color::Rgb { r: 0, g: 0, b: 0 },
                vec![
                    Color::Rgb { r: 255, g: 0, b: 0 },
                    Color::Rgb {
                        r: 255,
                        g: 127,
                        b: 0,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 0,
                    },
                    Color::Rgb { r: 0, g: 255, b: 0 },
                    Color::Rgb { r: 0, g: 0, b: 255 },
                    Color::Rgb {
                        r: 75,
                        g: 0,
                        b: 130,
                    },
                    Color::Rgb {
                        r: 148,
                        g: 0,
                        b: 211,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Aurora",
            palette: ThemePalette::new(
                Color::Rgb { r: 0, g: 5, b: 15 },
                vec![
                    Color::Rgb {
                        r: 0,
                        g: 50,
                        b: 100,
                    },
                    Color::Rgb {
                        r: 0,
                        g: 255,
                        b: 128,
                    },
                    Color::Rgb {
                        r: 0,
                        g: 255,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 128,
                        g: 0,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 0,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Galaxy",
            palette: ThemePalette::new(
                Color::Rgb { r: 5, g: 0, b: 15 },
                vec![
                    Color::Rgb { r: 25, g: 0, b: 50 },
                    Color::Rgb {
                        r: 75,
                        g: 0,
                        b: 130,
                    },
                    Color::Rgb {
                        r: 148,
                        g: 0,
                        b: 211,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 20,
                        b: 147,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 180,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Miami Vice",
            palette: ThemePalette::new(
                Color::Rgb { r: 0, g: 0, b: 0 },
                vec![
                    Color::Rgb {
                        r: 0,
                        g: 150,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 0,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 100,
                        b: 200,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 0,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Sunset",
            palette: ThemePalette::new(
                Color::Rgb { r: 10, g: 0, b: 20 },
                vec![
                    Color::Rgb { r: 50, g: 0, b: 50 },
                    Color::Rgb {
                        r: 150,
                        g: 0,
                        b: 100,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 50,
                        b: 50,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 150,
                        b: 0,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 100,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Oceanic",
            palette: ThemePalette::new(
                Color::Rgb { r: 0, g: 10, b: 30 },
                vec![
                    Color::Rgb {
                        r: 0,
                        g: 50,
                        b: 150,
                    },
                    Color::Rgb {
                        r: 0,
                        g: 102,
                        b: 204,
                    },
                    Color::Rgb {
                        r: 0,
                        g: 200,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 150,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Toxic",
            palette: ThemePalette::new(
                Color::Rgb { r: 5, g: 15, b: 5 },
                vec![
                    Color::Rgb {
                        r: 10,
                        g: 50,
                        b: 10,
                    },
                    Color::Rgb {
                        r: 50,
                        g: 205,
                        b: 50,
                    },
                    Color::Rgb {
                        r: 173,
                        g: 255,
                        b: 47,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Monochrome",
            palette: ThemePalette::new(
                Color::Rgb { r: 0, g: 0, b: 0 },
                vec![
                    Color::Rgb {
                        r: 20,
                        g: 20,
                        b: 20,
                    },
                    Color::Rgb {
                        r: 100,
                        g: 100,
                        b: 100,
                    },
                    Color::Rgb {
                        r: 200,
                        g: 200,
                        b: 200,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Cyberpunk",
            palette: ThemePalette::new(
                Color::Rgb { r: 10, g: 0, b: 20 },
                vec![
                    Color::Rgb {
                        r: 0,
                        g: 255,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 0,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 0,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Forest",
            palette: ThemePalette::new(
                Color::Rgb { r: 5, g: 10, b: 5 },
                vec![
                    Color::Rgb {
                        r: 20,
                        g: 50,
                        b: 10,
                    },
                    Color::Rgb {
                        r: 34,
                        g: 139,
                        b: 34,
                    },
                    Color::Rgb {
                        r: 107,
                        g: 142,
                        b: 35,
                    },
                    Color::Rgb {
                        r: 189,
                        g: 183,
                        b: 107,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Vampire",
            palette: ThemePalette::new(
                Color::Rgb { r: 5, g: 0, b: 0 },
                vec![
                    Color::Rgb { r: 50, g: 0, b: 0 },
                    Color::Rgb { r: 139, g: 0, b: 0 },
                    Color::Rgb {
                        r: 220,
                        g: 20,
                        b: 60,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 240,
                        b: 245,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Ice",
            palette: ThemePalette::new(
                Color::Rgb { r: 0, g: 5, b: 20 },
                vec![
                    Color::Rgb {
                        r: 0,
                        g: 50,
                        b: 150,
                    },
                    Color::Rgb {
                        r: 100,
                        g: 150,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 200,
                        g: 240,
                        b: 255,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Lava",
            palette: ThemePalette::new(
                Color::Rgb { r: 10, g: 0, b: 0 },
                vec![
                    Color::Rgb { r: 100, g: 0, b: 0 },
                    Color::Rgb {
                        r: 200,
                        g: 50,
                        b: 0,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 150,
                        b: 0,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 100,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Hacker",
            palette: ThemePalette::new(
                Color::Rgb { r: 0, g: 0, b: 0 },
                vec![
                    Color::Rgb { r: 0, g: 100, b: 0 },
                    Color::Rgb { r: 0, g: 255, b: 0 },
                    Color::Rgb {
                        r: 150,
                        g: 255,
                        b: 150,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Cotton Candy",
            palette: ThemePalette::new(
                Color::Rgb { r: 10, g: 0, b: 10 },
                vec![
                    Color::Rgb {
                        r: 255,
                        g: 105,
                        b: 180,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 182,
                        b: 193,
                    },
                    Color::Rgb {
                        r: 173,
                        g: 216,
                        b: 230,
                    },
                    Color::Rgb {
                        r: 240,
                        g: 255,
                        b: 255,
                    },
                ],
            ),
        },
        ThemeDef {
            name: "Gold",
            palette: ThemePalette::new(
                Color::Rgb { r: 10, g: 5, b: 0 },
                vec![
                    Color::Rgb {
                        r: 139,
                        g: 69,
                        b: 19,
                    },
                    Color::Rgb {
                        r: 218,
                        g: 165,
                        b: 32,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 215,
                        b: 0,
                    },
                    Color::Rgb {
                        r: 255,
                        g: 250,
                        b: 205,
                    },
                ],
            ),
        },
    ]
}
