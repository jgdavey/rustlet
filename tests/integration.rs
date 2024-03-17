extern crate encoding_rs;
extern crate encoding_rs_io;
extern crate rustlet;

mod diff;

use rustlet::font::Font;
use rustlet::{art_lines, read_font};

use std::fmt::Write;
use std::fs::File;
use std::io::{self, Error, ErrorKind, Read};
use std::path::Path;

use encoding_rs::UTF_8;
use encoding_rs_io::DecodeReaderBytesBuilder;

const FONTS: [&str; 18] = [
    "banner", "big", "block", "bubble", "digital", "ivrit", "lean", "mini", "mnemonic", "script",
    "shadow", "slant", "small", "smscript", "smshadow", "smslant", "standard", "term",
];

pub fn read_font_file<P: AsRef<Path>>(path: P) -> io::Result<Font> {
    let disp = format!("{}", path.as_ref().display());
    let file = File::open(path)?;
    let mut transcoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(UTF_8))
        .build(file);
    let mut out = String::new();
    transcoded.read_to_string(&mut out)?;
    //let out = fs::read_to_string(path)?;
    read_font(&out)
        .ok_or_else(|| Error::new(ErrorKind::Other, format!("Problem with path: {}", disp)))
}

#[test]
fn test_font_show() {
    let mut results = String::new();
    for font in FONTS {
        let path = format!("fonts/{}.flf", font);
        let parsed = read_font_file(path).expect("Font not read");
        let lines = art_lines(font, &parsed, &parsed.settings, 80);
        writeln!(results, "{} :", font).unwrap();
        for line in lines {
            writeln!(results, "{}", line).unwrap();
        }
        writeln!(results).unwrap();
        writeln!(results).unwrap();
    }
    let expected = include_str!("data/outputs/fontnames.txt");
    assert_diff!(expected, &results);
}
