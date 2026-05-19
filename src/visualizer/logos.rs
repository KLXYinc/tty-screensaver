use crate::buffer::ScreenBuffer;
use crate::charsets::CharSet;
use crate::themes::ThemePalette;
use crate::visualizer::Visualizer;
use crossterm::style::Color;

struct LogoDef {
    art: &'static str,
    color_fn: fn(f64, f64, char) -> Color,
}

fn color_windows(nx: f64, ny: f64, _c: char) -> Color {
    if nx < 0.38 && ny < 0.48 {
        Color::Rgb {
            r: 242,
            g: 80,
            b: 34,
        }
    } else if nx >= 0.38 && ny < 0.48 {
        Color::Rgb {
            r: 127,
            g: 186,
            b: 0,
        }
    } else if nx < 0.38 && ny >= 0.48 {
        Color::Rgb {
            r: 0,
            g: 164,
            b: 239,
        }
    } else {
        Color::Rgb {
            r: 255,
            g: 185,
            b: 0,
        }
    }
}

fn color_apple(_nx: f64, ny: f64, _c: char) -> Color {
    if ny < 0.23 {
        Color::Rgb {
            r: 97,
            g: 187,
            b: 70,
        }
    } else if ny < 0.40 {
        Color::Rgb {
            r: 253,
            g: 184,
            b: 39,
        }
    } else if ny < 0.55 {
        Color::Rgb {
            r: 245,
            g: 130,
            b: 31,
        }
    } else if ny < 0.70 {
        Color::Rgb {
            r: 224,
            g: 58,
            b: 62,
        }
    } else if ny < 0.85 {
        Color::Rgb {
            r: 150,
            g: 61,
            b: 151,
        }
    } else {
        Color::Rgb {
            r: 0,
            g: 157,
            b: 220,
        }
    }
}

fn color_arch(_nx: f64, _ny: f64, _c: char) -> Color {
    Color::Rgb {
        r: 23,
        g: 147,
        b: 209,
    }
}

fn color_ubuntu(nx: f64, ny: f64, _c: char) -> Color {
    let angle = (ny - 0.5).atan2(nx - 0.5);
    if angle < -0.5 && angle > -2.5 {
        Color::Rgb {
            r: 245,
            g: 130,
            b: 31,
        }
    } else if angle >= -0.5 && angle <= 1.5 {
        Color::Rgb {
            r: 253,
            g: 184,
            b: 39,
        }
    } else {
        Color::Rgb {
            r: 224,
            g: 58,
            b: 62,
        }
    }
}

fn color_android(_nx: f64, _ny: f64, _c: char) -> Color {
    Color::Rgb {
        r: 61,
        g: 220,
        b: 132,
    }
}

fn color_fedora(_nx: f64, _ny: f64, c: char) -> Color {
    match c {
        '-' | ':' | '/' | '\\' | ',' => Color::Rgb {
            r: 41,
            g: 65,
            b: 114,
        },
        _ => Color::White,
    }
}

const LOGOS: [LogoDef; 6] = [
    LogoDef {
        art: "                                ..,
                    ....,,:;+ccllll
      ...,,+:;  cllllllllllllllllll
,cclllllllllll  lllllllllllllllllll
llllllllllllll  lllllllllllllllllll
llllllllllllll  lllllllllllllllllll
llllllllllllll  lllllllllllllllllll
llllllllllllll  lllllllllllllllllll
llllllllllllll  lllllllllllllllllll
                                      
llllllllllllll  lllllllllllllllllll
llllllllllllll  lllllllllllllllllll
llllllllllllll  lllllllllllllllllll
llllllllllllll  lllllllllllllllllll
llllllllllllll  lllllllllllllllllll
`'ccllllllllll  lllllllllllllllllll
       `' \\*::  :ccllllllllllllllll
                       ````''*::cll
                                 ``",
        color_fn: color_windows,
    },
    LogoDef {
        art: "                    'c.
                 ,xNMM.
               .OMMMMo
               OMMM0,
     .;loddo:' loolloddol;.
   cKMMMMMMMMMMNWMMMMMMMMMM0:
 .KMMMMMMMMMMMMMMMMMMMMMMMWd.
 XMMMMMMMMMMMMMMMMMMMMMMMX.
;MMMMMMMMMMMMMMMMMMMMMMMM:
:MMMMMMMMMMMMMMMMMMMMMMMM:
.MMMMMMMMMMMMMMMMMMMMMMMMX.
 kMMMMMMMMMMMMMMMMMMMMMMMMWd.
 .XMMMMMMMMMMMMMMMMMMMMMMMMMMk
  .XMMMMMMMMMMMMMMMMMMMMMMMMK.
    kMMMMMMMMMMMMMMMMMMMMMMd
     ;KMMMMMMMWXXWMMMMMMMk.
       .cooc,.    .,coo:.",
        color_fn: color_apple,
    },
    LogoDef {
        art: "                   -`
                  .o+`
                 `ooo/
                `+oooo:
               `+oooooo:
               -+oooooo+:
             `/:-:++oooo+:
            `/++++/+++++++:
           `/++++++++++++++:
          `/+++ooooooooooooo/`
         ./ooosssso++osssssso+`
        .oossssso-````/ossssss+`
       -osssssso.      :ssssssso.
      :osssssss/        osssso+++.
     /ossssssss/        +ssssooo/-
   `/ossssso+/:-        -:/+osssso+-
  `+sso+:-`                 `.-/+oso:
 `++:.                           `-/+/
 .`                                 `/",
        color_fn: color_arch,
    },
    LogoDef {
        art: "            .-/+oossssoo+/-.
        `:+ssssssssssssssssss+:`
      -+ssssssssssssssssssyyssss+-
    .ossssssssssssssssssdMMMNysssso.
   /ssssssssssshdmmNNmmyNMMMMhssssss/
  +ssssssssshmydMMMMMMMNddddyssssssss+
 /sssssssshNMMMyhhyyyyhmNMMMNhssssssss/
.ssssssssdMMMNhsssssssssshNMMMdssssssss.
+sssssssNMMMysssssssssssssydnnmdsssssss+
ossssssyMMMyssssssssssssssssssssssssssso
ossssssyMMMyssssssssssssssssssssssssssso
+sssssssNMMMysssssssssssssydnnmdsssssss+
.ssssssssdMMMNhsssssssssshNMMMdssssssss.
 /sssssssshNMMMyhhyyyyhdNMMMNhssssssss/
  +sssssssssdmydMMMMMMMNddddyssssssss+
   /ssssssssssshdmmNNmmyNMMMMhssssss/
    .ossssssssssssssssssdMMMNysssso.
      -+sssssssssssssssssyyyssss+-
        `:+ssssssssssssssssss+:`
            .-/+oossssoo+/-.",
        color_fn: color_ubuntu,
    },
    LogoDef {
        art: "         -o          o-
          +hydNNNNdyh+
        +mMMMMMMMMMMMMm+
      `dMMm:NMMMMMMN:mMMd`
      hMMMMMMMMMMMMMMMMMMh
  ..  yyyyyyyyyyyyyyyyyyyy  ..
.mMMm `MMMMMMMMMMMMMMMMMM` mMMm.
:MMMM- `MMMMMMMMMMMMMMMMMM` -MMMM:
:MMMM- `MMMMMMMMMMMMMMMMMM` -MMMM:
:MMMM- `MMMMMMMMMMMMMMMMMM` -MMMM:
:MMMM- `MMMMMMMMMMMMMMMMMM` -MMMM:
-mMMm `MMMMMMMMMMMMMMMMMM` mMMm-
  ..  `MMMMMMMMMMMMMMMMMM`  ..
      `MMMMMMMMMMMMMMMMMM`
      `MMMMMMMMMMMMMMMMMM`
        -mMMMM-  -MMMMm-
         :MMMM:  :MMMM:
         -MMMM-  -MMMM-
          `--`    `--`",
        color_fn: color_android,
    },
    LogoDef {
        art: "          /:-------------:\\
       :-------------------::
     :-----------MshhOHbmp---:\\
   /-----------omMMMNNNMMD-----:
  :-----------sMMMMNMNMPM-------:
 :-----------:MMMdP---------------\\
,------------:MMMd-----------------:
:------------:MMMd------------------:
:--------oNMMMMMMMMMNho------------:
:-------MMshhhNNmac-MNm-------------:
:-----------.-NNN-------------------:
:------------MNMMMN---------------:
:------------dMMMMN-------------:
\\-----------dMMMMm------------/
 \\----------dMMMMb----------/
  \\-------oMMMMM----------/
   \\------------------/",
        color_fn: color_fedora,
    },
];

pub struct LogosVisualizer {
    width: u16,
    height: u16,
    palette: ThemePalette,
    charset: CharSet,
    accumulator: f64,
    speed_multiplier: f64,
    current_logo: usize,
    rotation: f64,
}

impl LogosVisualizer {
    pub fn new(width: u16, height: u16, palette: ThemePalette, charset: CharSet) -> Self {
        Self {
            width,
            height,
            palette,
            charset,
            accumulator: 0.0,
            speed_multiplier: 1.0,
            current_logo: 0,
            rotation: 0.0,
        }
    }
}

impl Visualizer for LogosVisualizer {
    fn update(&mut self, delta_time: f64) {
        let dt = delta_time * self.speed_multiplier;

        self.rotation += dt * 1.5;
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer) {
        if self.width != buffer.width || self.height != buffer.height {
            self.width = buffer.width;
            self.height = buffer.height;
        }
        buffer.clear();

        let raw_logo = LOGOS[self.current_logo].art.trim_matches('\n');
        let lines: Vec<&str> = raw_logo.split('\n').collect();

        let logo_h = lines.len() as f64;
        let logo_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f64;

        let aspect_ratio_correction = 1.0;

        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64 / 2.0;

        let scale_x = self.rotation.cos();
        let _scale_z = self.rotation.sin();

        let base_scale = (self.height as f64 * 0.5) / logo_h;

        for (ly, line) in lines.iter().enumerate() {
            let mut char_idx = 0;
            for c in line.chars() {
                if c != ' ' {
                    let ox = (char_idx as f64) - (logo_w / 2.0);
                    let oy = (ly as f64) - (logo_h / 2.0);

                    let rotated_x = (ox * aspect_ratio_correction) * scale_x;

                    let screen_x = (center_x + rotated_x * base_scale).round() as i32;
                    let screen_y = (center_y + oy * base_scale).round() as i32;

                    if screen_x >= 0
                        && screen_x < self.width as i32
                        && screen_y >= 0
                        && screen_y < self.height as i32
                    {
                        let nx = (char_idx as f64) / logo_w;
                        let ny = (ly as f64) / logo_h;
                        let color = (LOGOS[self.current_logo].color_fn)(nx, ny, c);

                        let draw_char = '█';

                        buffer.set(
                            screen_x as u16,
                            screen_y as u16,
                            draw_char,
                            color,
                            crossterm::style::Color::Reset,
                        );
                    }
                }
                char_idx += 1;
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
        if delta > 0 {
            self.current_logo = (self.current_logo + 1) % LOGOS.len();
        } else if delta < 0 {
            if self.current_logo == 0 {
                self.current_logo = LOGOS.len() - 1;
            } else {
                self.current_logo -= 1;
            }
        }
    }
}
