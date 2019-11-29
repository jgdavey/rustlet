#[macro_use]
extern crate bitflags;
extern crate nom;

mod font;
mod settings;
mod text;
use font::read_font;

fn main() {
    let rawfont = include_str!("../fonts/small.flf");
    let font = read_font(rawfont).unwrap();

    let message = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let glyphs: Vec<_> = message.chars().map(|c| font.get_character(&c)).collect();
    for line in 0..font.height() {
        for glyph in &glyphs {
            for ch in glyph.art[line].chars() {
                if font.is_hardblank(ch) {
                    print!("{}", ' ');
                } else {
                    print!("{}", ch);
                }
            }
        }
        println!();
    }
}
