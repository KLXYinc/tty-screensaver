use crate::app::Mode;
use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemeDef;
use crossterm::style::Color;

pub fn draw_hud(buffer: &mut ScreenBuffer, mode: Mode, theme: &ThemeDef, charset: &CharSet) {
    let text = format!(
        " Mode: {:<8} [◄/►] | Theme: {:<15} [▲/▼] | Charset: {:<10} [Ctrl+▲/▼] | Colors: ",
        mode.name(),
        theme.name,
        charset.name
    );

    for x in 0..buffer.width {
        buffer.set(x, 0, ' ', Color::White, Color::Black);
    }

    buffer.set_str(0, 0, &text, Color::White, Color::Black);

    let mut dots_x = text.len() as u16;
    for &color in &theme.palette.colors {
        if dots_x < buffer.width {
            buffer.set(dots_x, 0, '●', color, Color::Black);
            dots_x += 2;
        }
    }
    if dots_x < buffer.width {
        buffer.set(dots_x, 0, '●', theme.palette.background, Color::Black);
    }
}
