#[macro_use]
extern crate bitflags;
extern crate nom;
#[macro_use]
extern crate clap;

mod font;
mod settings;
mod text;

use clap::{App, Arg};
use font::{read_font, read_font_file};
//use settings::Settings;
use text::art_lines;

fn main() {
    let app = App::new("rustlet")
        .version(crate_version!())
        .about("ASCII art from messages")
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .value_name("WIDTH")
                .default_value("72")
                .help("Sets the maximum width for a line")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("font")
                .short("f")
                .long("font")
                .value_name("FONT")
                .default_value("standard")
                .help("Name of font file to use")
                .takes_value(true),
        )
        .arg(Arg::with_name("messages").multiple(true));

    let matches = app.get_matches();

    let rawfont = include_str!("../fonts/standard.flf");

    let font = matches
        .value_of("font")
        .and_then(|f| read_font_file(f).ok())
        .unwrap_or_else(|| read_font(rawfont).expect("Default font unreadable"));

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
        print!("{}", art_line);
    }
}
