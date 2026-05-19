pub mod aquarium;
pub mod boids;
pub mod bonsai;
pub mod breaker;
pub mod bubbles;
pub mod city3d;
pub mod clocks;
pub mod dvd;
pub mod earth;
pub mod fire;
pub mod hex3d;
pub mod life;
pub mod logos;
pub mod lorenz;
pub mod matrix;
pub mod maze;
pub mod metaballs;
pub mod minecraft;
pub mod name;
pub mod pacman;
pub mod perlin;
pub mod pingpong;
pub mod rain;
pub mod sand;
pub mod shape;
pub mod snake;
pub mod starfield;
pub mod stripes;
pub mod synthwave;
pub mod tetris;
pub mod waves;
use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemePalette;
pub trait Visualizer {
    fn update(&mut self, delta_time: f64);
    fn draw(&mut self, buffer: &mut ScreenBuffer);
    fn set_palette(&mut self, palette: ThemePalette);
    fn set_charset(&mut self, charset: CharSet);
    fn on_scroll(&mut self, _delta: i32) {}
    fn on_scroll_ext(&mut self, delta: i32, _is_ctrl: bool) {
        self.on_scroll(delta);
    }
    fn on_key(
        &mut self,
        _code: crossterm::event::KeyCode,
        _mods: crossterm::event::KeyModifiers,
    ) -> bool {
        false
    }
}
