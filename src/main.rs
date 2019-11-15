#[macro_use]
extern crate bitflags;
extern crate nom;

mod font;
use font::headerline;

fn main() {
    headerline("").unwrap();
}
