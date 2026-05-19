# TTY Screensaver

A high-performance, aesthetically pleasing terminal screensaver and visualizer engine written in Rust. Experience stunning generative art, 3D projections, and classic particle simulations directly in your terminal.

![TTY Screensaver Preview](./assets/preview.gif) *(Add your preview GIF here!)*

## 🌟 Features

- **30+ Unique Visualizers**: Includes digital rain (Matrix), Game of Life, Tetris, 3D projections (Earth, City3D, Cube), particle physics (Sand, Rain, Fire), and much more.
- **Dynamic Themes**: Beautiful, curated color palettes such as `Cyberpunk`, `Forest`, `Vampire`, `Synthwave`, `Miami Vice`, and `Default Matrix`.
- **Custom Character Sets**: Instantly swap the rendering glyphs to `Runes`, `Math`, `Greek`, `Cyrillic`, `Emoji`, or `Braille`.
- **Blazing Fast**: Written in pure Rust using `crossterm` for maximum performance and zero stuttering.
- **Cross-Platform**: Runs seamlessly on Linux, Windows, and macOS natively.

## 📸 Gallery

*(Replace these placeholders with actual screenshots!)*

| Matrix Mode | City3D Mode | Cyberpunk Theme |
|:---:|:---:|:---:|
| `![Matrix](./assets/matrix.png)` | `![City3D](./assets/city3d.png)` | `![Cyberpunk](./assets/cyberpunk.png)` |

## 🚀 Installation

Ensure you have the [Rust toolchain](https://rustup.rs/) installed on your machine.

Clone the repository and build the project natively:

```bash
git clone https://github.com/KLXYinc/tty-screensaver.git
cd tty-screensaver
cargo build --release
```

The optimized binary will be generated at `target/release/tty-screensaver`.

## 💻 Usage

Run the compiled binary in your favorite terminal:

```bash
./target/release/tty-screensaver
```

- **Scroll Up/Down**: Seamlessly cycle through the 30+ visualizer modes.
- **Esc / Ctrl+C**: Quit the application.
