#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    unused_mut,
    unused_assignments
)]

pub mod app;
mod buffer;
mod charsets;
mod config;
mod themes;
mod ui;
mod utils;
mod visualizer;

use clap::Parser;

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, size,
    },
};
use std::io::stdout;

use app::App;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 60)]
    fps: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;
    enable_raw_mode()?;

    let (width, height) = size()?;

    let mut app = App::new(args.fps, width, height);
    app.run(&mut stdout)?;

    disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen)?;

    Ok(())
}
