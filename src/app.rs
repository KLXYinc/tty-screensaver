use crate::buffer::ScreenBuffer;
use crate::charsets::{CharSet, get_all_charsets_utf8};
use crate::config::AppConfig;
use crate::themes::{ThemeDef, ThemePalette, get_all_themes};
use crate::ui::draw_hud;
use crate::visualizer::Visualizer;
use crate::visualizer::aquarium::AquariumVisualizer;
use crate::visualizer::boids::BoidsVisualizer;
use crate::visualizer::bonsai::BonsaiVisualizer;
use crate::visualizer::breaker::BreakerVisualizer;
use crate::visualizer::bubbles::BubblesVisualizer;
use crate::visualizer::city3d::City3DVisualizer;
use crate::visualizer::clocks::ClocksVisualizer;
use crate::visualizer::dvd::DvdVisualizer;
use crate::visualizer::earth::EarthVisualizer;
use crate::visualizer::fire::FireVisualizer;
use crate::visualizer::hex3d::Hex3DVisualizer;
use crate::visualizer::life::LifeVisualizer;
use crate::visualizer::logos::LogosVisualizer;
use crate::visualizer::lorenz::LorenzVisualizer;
use crate::visualizer::matrix::MatrixVisualizer;
use crate::visualizer::maze::MazeVisualizer;
use crate::visualizer::metaballs::MetaballsVisualizer;
use crate::visualizer::minecraft::MinecraftVisualizer;
use crate::visualizer::name::NameVisualizer;
use crate::visualizer::pacman::PacmanVisualizer;
use crate::visualizer::perlin::PerlinVisualizer;
use crate::visualizer::pingpong::PingPongVisualizer;
use crate::visualizer::rain::RainVisualizer;
use crate::visualizer::sand::SandVisualizer;
use crate::visualizer::shape::ShapeVisualizer;
use crate::visualizer::snake::SnakeVisualizer;
use crate::visualizer::starfield::StarfieldVisualizer;
use crate::visualizer::stripes::StripesVisualizer;
use crate::visualizer::synthwave::SynthwaveVisualizer;
use crate::visualizer::tetris::TetrisVisualizer;
use crate::visualizer::waves::WavesVisualizer;
use crossterm::{
    cursor::MoveTo,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, poll,
        read,
    },
    execute, queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{Stdout, Write};
use std::time::{Duration, Instant};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Matrix,
    Fire,
    Perlin,
    Starfield,
    Rain,
    Shape,
    Life,
    Sand,
    Pacman,
    Snake,
    Earth,
    Waves,
    Tetris,
    Maze,
    Stripes,
    Bubbles,
    Logos,
    Synthwave,
    Dvd,
    PingPong,
    Breaker,
    City3D,
    Boids,
    Metaballs,
    Lorenz,
    Hex3D,
    Minecraft,
    Clocks,
    Aquarium,
    Name,
    Bonsai,
}
impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Matrix => "Matrix",
            Mode::Fire => "Fire",
            Mode::Perlin => "Perlin",
            Mode::Starfield => "Stars",
            Mode::Rain => "Rain",
            Mode::Shape => "Shape",
            Mode::Life => "Life",
            Mode::Sand => "Sand",
            Mode::Pacman => "Pacman",
            Mode::Snake => "Snake",
            Mode::Earth => "Earth",
            Mode::Waves => "Waves",
            Mode::Tetris => "Tetris",
            Mode::Maze => "Maze",
            Mode::Stripes => "Stripes",
            Mode::Bubbles => "Bubbles",
            Mode::Logos => "Logos",
            Mode::Synthwave => "Synthwave",
            Mode::Dvd => "DVD",
            Mode::PingPong => "PingPong",
            Mode::Breaker => "Breaker",
            Mode::City3D => "City3D",
            Mode::Boids => "Boids",
            Mode::Metaballs => "Metaballs",
            Mode::Lorenz => "Lorenz",
            Mode::Hex3D => "Hex3D",
            Mode::Minecraft => "Minecraft",
            Mode::Clocks => "Clocks",
            Mode::Aquarium => "Aquarium",
            Mode::Name => "Name",
            Mode::Bonsai => "Bonsai",
        }
    }
}
pub const MODES: [Mode; 31] = [
    Mode::Matrix,
    Mode::Fire,
    Mode::Perlin,
    Mode::Starfield,
    Mode::Rain,
    Mode::Shape,
    Mode::Life,
    Mode::Sand,
    Mode::Pacman,
    Mode::Snake,
    Mode::Earth,
    Mode::Waves,
    Mode::Tetris,
    Mode::Maze,
    Mode::Stripes,
    Mode::Bubbles,
    Mode::Logos,
    Mode::Synthwave,
    Mode::Dvd,
    Mode::PingPong,
    Mode::Breaker,
    Mode::City3D,
    Mode::Boids,
    Mode::Metaballs,
    Mode::Lorenz,
    Mode::Hex3D,
    Mode::Minecraft,
    Mode::Clocks,
    Mode::Aquarium,
    Mode::Name,
    Mode::Bonsai,
];
pub struct App {
    mode_idx: usize,
    config: AppConfig,
    themes: Vec<ThemeDef>,
    charsets: Vec<CharSet>,
    visualizer: Box<dyn Visualizer>,
    current_buffer: ScreenBuffer,
    prev_buffer: ScreenBuffer,
    width: u16,
    height: u16,
    target_frame_duration: Duration,
    hud_timer: f64,
}
impl App {
    pub fn new(fps: u32, width: u16, height: u16) -> Self {
        let themes = get_all_themes();
        let charsets = get_all_charsets_utf8();
        let config = AppConfig::load();
        let mut mode_idx = 0;
        if !config.last_mode.is_empty() {
            for (i, mode) in MODES.iter().enumerate() {
                if mode.name() == config.last_mode {
                    mode_idx = i;
                    break;
                }
            }
        }
        let mode = MODES[mode_idx];
        let mode_conf = config.get_mode_config(mode.name());
        let theme_idx = mode_conf.theme_idx.min(themes.len() - 1);
        let charset_idx = mode_conf.charset_idx.min(charsets.len() - 1);
        let visualizer = Self::create_visualizer(
            mode,
            width,
            height,
            themes[theme_idx].palette.clone(),
            charsets[charset_idx].clone(),
            config.utc_offset_hours,
        );
        Self {
            mode_idx,
            config,
            themes,
            charsets,
            visualizer,
            current_buffer: ScreenBuffer::new(width, height),
            prev_buffer: ScreenBuffer::new(width, height),
            width,
            height,
            target_frame_duration: Duration::from_secs_f64(1.0 / fps as f64),
            hud_timer: 3.0,
        }
    }
    fn create_visualizer(
        mode: Mode,
        width: u16,
        height: u16,
        palette: ThemePalette,
        charset: CharSet,
        utc_offset_hours: i32,
    ) -> Box<dyn Visualizer> {
        match mode {
            Mode::Matrix => Box::new(MatrixVisualizer::new(width, height, palette, charset)),
            Mode::Fire => Box::new(FireVisualizer::new(width, height, palette, charset)),
            Mode::Perlin => Box::new(PerlinVisualizer::new(width, height, palette, charset)),
            Mode::Starfield => Box::new(StarfieldVisualizer::new(width, height, palette, charset)),
            Mode::Rain => Box::new(RainVisualizer::new(width, height, palette, charset)),
            Mode::Shape => Box::new(ShapeVisualizer::new(width, height, palette, charset)),
            Mode::Life => Box::new(LifeVisualizer::new(width, height, palette, charset)),
            Mode::Sand => Box::new(SandVisualizer::new(width, height, palette, charset)),
            Mode::Pacman => Box::new(PacmanVisualizer::new(width, height, palette, charset)),
            Mode::Snake => Box::new(SnakeVisualizer::new(width, height, palette, charset)),
            Mode::Earth => Box::new(EarthVisualizer::new(width, height, palette, charset)),
            Mode::Waves => Box::new(WavesVisualizer::new(width, height, palette, charset)),
            Mode::Tetris => Box::new(TetrisVisualizer::new(width, height, palette, charset)),
            Mode::Maze => Box::new(MazeVisualizer::new(width, height, palette, charset)),
            Mode::Stripes => Box::new(StripesVisualizer::new(width, height, palette, charset)),
            Mode::Bubbles => Box::new(BubblesVisualizer::new(width, height, palette, charset)),
            Mode::Logos => Box::new(LogosVisualizer::new(width, height, palette, charset)),
            Mode::Synthwave => Box::new(SynthwaveVisualizer::new(width, height, palette, charset)),
            Mode::Dvd => Box::new(DvdVisualizer::new(width, height, palette, charset)),
            Mode::PingPong => Box::new(PingPongVisualizer::new(width, height, palette, charset)),
            Mode::Breaker => Box::new(BreakerVisualizer::new(width, height, palette, charset)),
            Mode::City3D => Box::new(City3DVisualizer::new(width, height, palette, charset)),
            Mode::Boids => Box::new(BoidsVisualizer::new(width, height, palette, charset)),
            Mode::Metaballs => Box::new(MetaballsVisualizer::new(width, height, palette, charset)),
            Mode::Lorenz => Box::new(LorenzVisualizer::new(width, height, palette, charset)),
            Mode::Hex3D => Box::new(Hex3DVisualizer::new(width, height, palette, charset)),
            Mode::Minecraft => Box::new(MinecraftVisualizer::new(width, height, palette, charset)),
            Mode::Clocks => Box::new(ClocksVisualizer::new(
                width,
                height,
                palette,
                charset,
                utc_offset_hours,
            )),
            Mode::Aquarium => Box::new(AquariumVisualizer::new(width, height, palette, charset)),
            Mode::Name => Box::new(NameVisualizer::new(width, height, palette, charset)),
            Mode::Bonsai => Box::new(BonsaiVisualizer::new(width, height, palette, charset)),
        }
    }
    pub fn run(&mut self, stdout: &mut Stdout) -> Result<(), Box<dyn std::error::Error>> {
        execute!(stdout, EnableMouseCapture)?;
        execute!(stdout, ResetColor, Clear(ClearType::All))?;
        let mut last_frame_time = Instant::now();
        'main_loop: loop {
            while poll(Duration::from_millis(0))? {
                let event = read()?;
                match event {
                    Event::Key(key_event) => {
                        if key_event.kind != KeyEventKind::Press {
                            continue;
                        }
                        let mut changed_theme = false;
                        let mut changed_mode = false;
                        let mut changed_charset = false;
                        let consumed = self.visualizer.on_key(key_event.code, key_event.modifiers);
                        let mode_name = MODES[self.mode_idx].name();
                        let mut mode_conf = self.config.get_mode_config(mode_name);
                        if !consumed {
                            match key_event.code {
                                KeyCode::Char('q') | KeyCode::Esc => break 'main_loop,
                                KeyCode::Up
                                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    mode_conf.charset_idx =
                                        (mode_conf.charset_idx + 1) % self.charsets.len();
                                    changed_charset = true;
                                }
                                KeyCode::Down
                                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    mode_conf.charset_idx =
                                        (mode_conf.charset_idx + self.charsets.len() - 1)
                                            % self.charsets.len();
                                    changed_charset = true;
                                }
                                KeyCode::Right => {
                                    self.mode_idx = (self.mode_idx + 1) % MODES.len();
                                    changed_mode = true;
                                }
                                KeyCode::Left => {
                                    self.mode_idx = (self.mode_idx + MODES.len() - 1) % MODES.len();
                                    changed_mode = true;
                                }
                                KeyCode::Up => {
                                    mode_conf.theme_idx =
                                        (mode_conf.theme_idx + 1) % self.themes.len();
                                    changed_theme = true;
                                }
                                KeyCode::Down => {
                                    mode_conf.theme_idx = (mode_conf.theme_idx + self.themes.len()
                                        - 1)
                                        % self.themes.len();
                                    changed_theme = true;
                                }
                                _ => {}
                            }
                        }
                        if changed_theme || changed_charset {
                            self.config.set_mode_config(
                                mode_name,
                                mode_conf.theme_idx,
                                mode_conf.charset_idx,
                            );
                        }
                        if changed_mode || changed_theme || changed_charset {
                            self.hud_timer = 3.0;
                        }
                        if changed_mode {
                            execute!(stdout, ResetColor, Clear(ClearType::All))?;
                            self.prev_buffer.clear();
                            let new_mode = MODES[self.mode_idx];
                            self.config.last_mode = new_mode.name().to_string();
                            self.config.save();
                            let new_conf = self.config.get_mode_config(new_mode.name());
                            let theme_idx = new_conf.theme_idx.min(self.themes.len() - 1);
                            let charset_idx = new_conf.charset_idx.min(self.charsets.len() - 1);
                            self.visualizer = Self::create_visualizer(
                                new_mode,
                                self.width,
                                self.height,
                                self.themes[theme_idx].palette.clone(),
                                self.charsets[charset_idx].clone(),
                                self.config.utc_offset_hours,
                            );
                        } else {
                            if changed_theme {
                                self.visualizer
                                    .set_palette(self.themes[mode_conf.theme_idx].palette.clone());
                            }
                            if changed_charset {
                                self.visualizer
                                    .set_charset(self.charsets[mode_conf.charset_idx].clone());
                            }
                        }
                    }
                    Event::Mouse(mouse_event) => {
                        let delta = match mouse_event.kind {
                            crossterm::event::MouseEventKind::ScrollUp => 1,
                            crossterm::event::MouseEventKind::ScrollDown => -1,
                            _ => 0,
                        };
                        if delta != 0 {
                            let is_ctrl = mouse_event.modifiers.contains(KeyModifiers::CONTROL);
                            self.visualizer.on_scroll_ext(delta, is_ctrl);
                            self.hud_timer = 3.0;
                        }
                    }
                    Event::Resize(new_width, new_height) => {
                        self.width = new_width;
                        self.height = new_height;
                        self.current_buffer.resize(new_width, new_height);
                        self.prev_buffer.resize(new_width, new_height);
                        self.hud_timer = 3.0;
                        execute!(stdout, ResetColor, Clear(ClearType::All))?;
                    }
                    _ => {}
                }
            }
            let now = Instant::now();
            let delta_time = now.duration_since(last_frame_time).as_secs_f64();
            last_frame_time = now;
            self.visualizer.update(delta_time);
            if self.hud_timer > 0.0 {
                self.hud_timer -= delta_time;
            }
            self.visualizer.draw(&mut self.current_buffer);
            if self.hud_timer > 0.0 {
                let mode_name = MODES[self.mode_idx].name();
                let conf = self.config.get_mode_config(mode_name);
                let t_idx = conf.theme_idx.min(self.themes.len() - 1);
                let c_idx = conf.charset_idx.min(self.charsets.len() - 1);
                draw_hud(
                    &mut self.current_buffer,
                    MODES[self.mode_idx],
                    &self.themes[t_idx],
                    &self.charsets[c_idx],
                );
            }
            self.render_diff(stdout)?;
            let elapsed = now.elapsed();
            if elapsed < self.target_frame_duration {
                std::thread::sleep(self.target_frame_duration - elapsed);
            }
        }
        execute!(stdout, DisableMouseCapture)?;
        Ok(())
    }
    fn render_diff(&mut self, stdout: &mut Stdout) -> std::io::Result<()> {
        let mut last_fg = None;
        let mut last_bg = None;
        let mut last_y = u16::MAX;
        let mut last_x = u16::MAX;
        for y in 0..self.height {
            for x in 0..self.width {
                let current_cell = self.current_buffer.get(x, y).unwrap();
                let prev_cell = self.prev_buffer.get(x, y).unwrap();
                if current_cell != prev_cell {
                    if y != last_y || x != last_x + 1 {
                        queue!(stdout, MoveTo(x, y))?;
                    }
                    last_x = x;
                    last_y = y;
                    if Some(current_cell.fg) != last_fg {
                        queue!(stdout, SetForegroundColor(current_cell.fg))?;
                        last_fg = Some(current_cell.fg);
                    }
                    if Some(current_cell.bg) != last_bg {
                        queue!(stdout, SetBackgroundColor(current_cell.bg))?;
                        last_bg = Some(current_cell.bg);
                    }
                    queue!(stdout, Print(current_cell.c))?;
                }
            }
        }
        stdout.flush()?;
        std::mem::swap(&mut self.current_buffer, &mut self.prev_buffer);
        Ok(())
    }
}
