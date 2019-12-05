#[macro_use]
extern crate bitflags;
extern crate nom;
#[macro_use]
extern crate clap;

mod font;
mod settings;
mod text;

use clap::{App, Arg};
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
    let app = App::new("rustlet")
        .version(crate_version!())
        .about("ASCII art from messages")
        .arg(Arg::with_name("width")
             .short("w")
             .long("width")
             .value_name("WIDTH")
             .default_value("72")
             .help("Sets the maximum width for a line")
             .takes_value(true))
        // .arg(Arg::with_name("font")
        //      .short("f")
        //      .long("font")
        //      .value_name("FONT")
        //      .default_value("standard")
        //      .help("Name of font file to use")
        //      .takes_value(true))
        .arg(Arg::with_name("messages")
             .multiple(true));

    let matches = app.get_matches();

    let rawfont = include_str!("../fonts/small.flf");
    let font = read_font(rawfont).unwrap();

    let max_size: usize = matches
        .value_of("width")
        .unwrap()
        .parse()
        .expect("Choose a valid positive number for width");

    let message = matches
        .values_of("messages")
        .expect("Provide some text to format")
        .collect::<Vec<_>>()
        .join(" ");

    let art_lines = art_lines(&message, &font, &font.settings, max_size);

    for art_line in art_lines {
        let block = art_line.to_string().replace(font.hardblank(), " ");
        print!("{}", block);
    }
}
