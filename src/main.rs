#[macro_use]
extern crate bitflags;
extern crate nom;

mod font;
mod settings;
mod text;
use font::read_font;
use text::Text;

fn main() {
    let rawfont = include_str!("../fonts/small.flf");
    let font = read_font(rawfont).unwrap();

    let message = std::env::args().skip(1).collect::<Vec<_>>().join(" ");

    let mut result = Text::empty_of_height(font.settings.charheight);

    for ch in message.chars().map(|c| font.get_character(&c)) {
        result.append(ch, &font.settings).unwrap();
    }

    for line in &result.art {
        for &ch in line.iter() {
            if font.is_hardblank(ch) {
                print!("{}", ' ');
            } else {
                print!("{}", ch);
            }
        }
        println!();
    }
}
