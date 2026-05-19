use crossterm::style::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
        }
    }
}

pub struct ScreenBuffer {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
}

impl ScreenBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); (width as usize) * (height as usize)],
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.cells = vec![Cell::default(); (width as usize) * (height as usize)];
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn set(&mut self, x: u16, y: u16, c: char, fg: Color, bg: Color) {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.cells[idx] = Cell { c, fg, bg };
        }
    }

    pub fn set_str(&mut self, x: u16, y: u16, text: &str, fg: Color, bg: Color) {
        let mut curr_x = x;
        for c in text.chars() {
            if curr_x >= self.width {
                break;
            }
            self.set(curr_x, y, c, fg, bg);
            curr_x += 1;
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        if x < self.width && y < self.height {
            Some(&self.cells[(y as usize) * (self.width as usize) + (x as usize)])
        } else {
            None
        }
    }
}
