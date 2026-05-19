use crate::buffer::ScreenBuffer;
use crossterm::style::Color;
pub struct ZBuffer {
    data: Vec<f64>,
    pub width: u16,
    pub height: u16,
}
impl ZBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            data: vec![f64::INFINITY; width as usize * height as usize],
            width,
            height,
        }
    }
    pub fn clear(&mut self) {
        self.data.fill(f64::INFINITY);
    }
    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.data = vec![f64::INFINITY; width as usize * height as usize];
        }
    }
    pub fn try_set(
        &mut self,
        x: i32,
        y: i32,
        depth: f64,
        buffer: &mut ScreenBuffer,
        ch: char,
        fg: Color,
    ) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        let idx = y as usize * self.width as usize + x as usize;
        if depth < self.data[idx] {
            self.data[idx] = depth;
            buffer.set(x as u16, y as u16, ch, fg, Color::Reset);
            return true;
        }
        false
    }
}
