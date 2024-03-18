#[macro_use]
extern crate bitflags;
extern crate nom;

mod font;
mod settings;
mod text;

use font::{read_font, read_font_file, Font};
//use settings::Settings;
use text::art_lines;

use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Message to convert to ascii art
    message: Vec<String>,

    /// Set a custom font directory
    #[arg(short = 'd', long = "font-directory", value_name = "DIR")]
    fontdir: Option<PathBuf>,

    /// Set max-width to value
    #[arg(short, long, default_value_t = 80)]
    width: u16,

    /// Set the font
    #[arg(short = 'f', long = "font", value_name = "FONT")]
    font: Option<String>,
}

fn find_font_dir() -> Option<PathBuf> {
    for dir in [
        "/usr/share/figlet/fonts",
        "/usr/local/share/figlet/fonts",
        "/opt/homebrew/share/figlet/fonts",
    ] {
        let d = PathBuf::from(dir);
        if d.is_dir() {
            return Some(d);
        }
    }
    None
}

fn read_font_from(dir: &Path, font: &str) -> Option<Font> {
    for ext in ["flf", "tlf"] {
        let mut base = PathBuf::from(font);
        base.set_extension(ext);
        let path = dir.join(base);
        // println!("path: {}", path.to_string_lossy());
        if let Ok(f) = read_font_file(&path) {
            return Some(f);
        }
    }
    None
}

fn main() {
    let cli = Cli::parse();

    let rawfont = include_str!("../fonts/standard.flf");

    let fontdir = cli
        .fontdir
        .or_else(find_font_dir)
        .unwrap_or(PathBuf::from("fonts"));

    let font = cli
        .font
        .and_then(|f| read_font_from(&fontdir, &f))
        .or_else(|| read_font(rawfont))
        .expect("No font readable");

    let max_size: usize = cli.width as usize;

    let message = cli.message.join(" ");

    let art_lines = art_lines(&message, &font, &font.settings, max_size);

    for art_line in art_lines {
        print!("{}", art_line);
    }
}
