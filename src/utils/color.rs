use crossterm::style::Color;
pub fn dim_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb { r, g, b } => Color::Rgb {
            r: (r as f32 * factor).clamp(0.0, 255.0) as u8,
            g: (g as f32 * factor).clamp(0.0, 255.0) as u8,
            b: (b as f32 * factor).clamp(0.0, 255.0) as u8,
        },
        other => other,
    }
}
pub fn blend_colors(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (r1, g1, b1) = rgb_components(a);
    let (r2, g2, b2) = rgb_components(b);
    Color::Rgb {
        r: (r1 + (r2 - r1) * t) as u8,
        g: (g1 + (g2 - g1) * t) as u8,
        b: (b1 + (b2 - b1) * t) as u8,
    }
}
fn rgb_components(c: Color) -> (f32, f32, f32) {
    match c {
        Color::Rgb { r, g, b } => (r as f32, g as f32, b as f32),
        _ => (255.0, 255.0, 255.0),
    }
}
