extern crate encoding_rs;
extern crate encoding_rs_io;
extern crate rustlet;

mod diff;

use rustlet::art_lines;
use rustlet::font::read_font_file;

use std::fmt::Write;

const FONTS: [&str; 18] = [
    "banner", "big", "block", "bubble", "digital", "ivrit", "lean", "mini", "mnemonic", "script",
    "shadow", "slant", "small", "smscript", "smshadow", "smslant", "standard", "term",
];

#[test]
fn test_font_show() {
    let mut results = String::new();
    for font in FONTS {
        let path = format!("fonts/{}.flf", font);
        let parsed = read_font_file(path).expect("Font not read");
        let lines = art_lines(font, &parsed, &parsed.settings, 80);
        writeln!(results, "{} :", font).unwrap();
        for line in lines {
            write!(results, "{}", line).unwrap();
        }
        writeln!(results).unwrap();
        writeln!(results).unwrap();
    }
    let expected = include_str!("data/outputs/fontnames.txt");
    assert_diff!(expected, &results);
}

#[test]
fn test_standard() {
    let parsed = read_font_file("fonts/standard.flf").expect("Font not read");
    let mut results = String::new();
    let lines = art_lines("take me home", &parsed, &parsed.settings, 80);
    for line in lines {
        write!(results, "{}", line).unwrap()
    }
    let expected = include_str!("data/outputs/takemehome.txt");
    assert_diff!(expected, &results);
}

#[test]
fn test_lean() {
    let parsed = read_font_file("fonts/lean.flf").expect("Font not read");
    let mut results = String::new();
    let lines = art_lines("lean on me", &parsed, &parsed.settings, 80);
    for line in lines {
        write!(results, "{}", line).unwrap()
    }
    let expected = include_str!("data/outputs/leanonme.txt");
    assert_diff!(expected, &results);
}
