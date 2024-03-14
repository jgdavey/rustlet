#[macro_use]
extern crate bitflags;
extern crate nom;
#[macro_use]
extern crate clap;

use std::fs;

mod font;
mod settings;
mod text;

use clap::{App, Arg};
use font::read_font;
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

    let rawfont = include_str!("../fonts/small.flf");

    let fontcontents = matches
        .value_of("font")
        .and_then(|f| fs::read_to_string(f).ok())
        .unwrap_or_else(|| rawfont.to_string());

    let font = read_font(&fontcontents).unwrap();

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
