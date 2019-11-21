#[macro_use]
extern crate bitflags;
extern crate nom;

mod font;
mod settings;
use font::headerline;

fn main() {
    headerline("").unwrap();
}
