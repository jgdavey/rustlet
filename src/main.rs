#[macro_use]
extern crate bitflags;
extern crate nom;

mod font;
mod settings;
mod text;

use font::{read_font, Font};
use settings::Settings;
use text::Text;

fn art_lines(message: &str, font: &Font, settings: &Settings, max_width: usize) -> Vec<Text> {
    let space = font.get_character(&' ');
    let words: Vec<Text> = message
        .split_whitespace()
        .flat_map(|word| {
            let mut result = vec![];
            let mut line = Text::empty_of_height(font.height());
            for ch in word.chars().map(|c| font.get_character(&c)) {
                let new_width = line.width() + ch.width();
                if !line.is_empty() && new_width > max_width {
                    result.push(line);
                    line = Text::empty_of_height(font.height());
                }
                line = line.append(&ch, settings);
            }
            if !line.is_empty() {
                result.push(line);
            }
            result.into_iter()
        })
        .collect();

    let mut result = vec![];
    let mut line = Text::empty_of_height(font.height());

    for word in words {
        let new_width = line.width() + word.width() + space.width();
        if !line.is_empty() {
            if new_width > max_width {
                result.push(line);
                line = Text::empty_of_height(font.height());
            } else {
                line = line.append(&space, settings);
            }
        }
        line = line.append(&word, settings);
    }

    if !line.is_empty() {
        result.push(line);
    }

    result
}

fn main() {
    let rawfont = include_str!("../fonts/small.flf");
    let font = read_font(rawfont).unwrap();

    let message = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let max_size: usize = 70;

    let art_lines = art_lines(&message, &font, &font.settings, max_size);

    for art_line in art_lines {
        let block = art_line.to_string().replace(font.hardblank(), " ");
        print!("{}", block);
    }
}
