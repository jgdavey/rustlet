use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{anychar, digit1, line_ending, space0},
    combinator::{map, map_opt, map_res, opt, recognize},
    multi::{many0, many_m_n},
    sequence::{delimited, pair, terminated, tuple},
    IResult,
};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

use crate::settings::{Settings, SmushMode};
use crate::text::Text;

use encoding_rs::UTF_8;
use encoding_rs_io::DecodeReaderBytesBuilder;

use std::fs::File;
use std::io::{self, Error, ErrorKind, Read};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Font {
    pub settings: Settings,
    pub comment: String,
    characters: HashMap<char, Text>,
}

impl Font {
    pub fn get_character(&self, ch: &char) -> &Text {
        self.characters
            .get(ch)
            .unwrap_or_else(|| self.characters.get(&'?').unwrap())
    }
    pub fn height(&self) -> u32 {
        self.settings.charheight
    }
    pub fn hardblank(&self) -> char {
        self.settings.hardblank
    }
}

fn delimited_i32(input: &str) -> IResult<&str, i32> {
    map_res(
        delimited(space0, recognize(pair(opt(tag("-")), digit1)), space0),
        FromStr::from_str,
    )(input)
}

fn delimited_u32(input: &str) -> IResult<&str, u32> {
    map_res(delimited(space0, digit1, space0), FromStr::from_str)(input)
}

fn settingsline(input: &str) -> IResult<&str, Settings> {
    let (
        input,
        (
            _,
            hardblank,
            charheight,
            baseline,
            maxlength,
            smush,
            commentlines,
            right2left,
            smush2,
            _codetags,
        ),
    ) = tuple((
        alt((tag("tlf2a"), tag("flf2a"))),
        anychar,
        delimited_u32,
        delimited_u32,
        delimited_u32,
        delimited_i32,
        delimited_u32,
        opt(delimited_u32),
        opt(delimited_u32),
        opt(delimited_u32),
    ))(input)?;

    let smushmode = if let Some(sm) = smush2 {
        SmushMode::from(sm)
    } else {
        SmushMode::from_old_layout(smush)
    };

    Ok((
        input,
        Settings {
            hardblank,
            charheight,
            baseline,
            maxlength,
            commentlines,
            right2left: right2left.unwrap_or(0) != 0,
            smushmode,
        },
    ))
}

fn non_line_ending(c: char) -> bool {
    c != '\n' && c != '\r'
}

fn parse_code_tag(input: &str) -> IResult<&str, (char, String)> {
    tuple((
        map_opt(delimited_u32, std::char::from_u32),
        map(line, String::from),
    ))(input)
}

/// Reads a single line of input, returning without the newline character
fn line(input: &str) -> IResult<&str, &str> {
    terminated(take_while(non_line_ending), line_ending)(input)
}

fn trim_line(line: &str) -> VecDeque<char> {
    if line.len() < 2 {
        VecDeque::new()
    } else {
        let mut chars: VecDeque<_> = line.chars().collect();
        let tailchar = chars.pop_back();
        while chars.back() == tailchar.as_ref() {
            chars.pop_back();
        }
        chars
    }
}

fn parse_font(input: &str) -> IResult<&str, Font> {
    let (input, settings) = terminated(settingsline, line_ending)(input)?;
    let comlines = settings.commentlines as usize;
    let height = settings.charheight as usize;
    let (input, comment) =
        map(many_m_n(comlines, comlines, line), |lines| lines.join("\n"))(input)?;
    let parse_character = map(many_m_n(height, height, line), |ch| {
        ch.into_iter().map(trim_line).collect::<Vec<_>>()
    });
    let parse_additional_character = tuple((parse_code_tag, &parse_character));

    let (input, required_characters) = many_m_n(102, 102, &parse_character)(input)?;

    let mut characters = HashMap::new();

    // standard ascii
    for (i, c) in (32u8..127).map(|i| i as char).enumerate() {
        let character = Text {
            text: c.to_string(),
            art: required_characters[i].clone(),
        };
        characters.insert(c, character);
    }

    // additional required (German) characters
    for (i, c) in [196u8, 214, 22, 228, 246, 252, 223]
        .iter()
        .map(|i| *i as char)
        .enumerate()
    {
        let character = Text {
            text: c.to_string(),
            art: required_characters[i].clone(),
        };
        characters.insert(c, character);
    }

    //println!("CHARACTERS: {:?}", characters);
    // what's left is additional characters
    let (input, additional_characters) = many0(parse_additional_character)(input)?;

    //println!("Additional characters: {:#?}", additional_characters);

    for ((c, _comment), art) in additional_characters {
        let character = Text {
            text: c.to_string(),
            art,
        };
        characters.insert(c, character);
    }

    Ok((
        input,
        Font {
            settings,
            comment,
            characters,
        },
    ))
}

pub fn read_font(font_data: &str) -> Option<Font> {
    if let Ok((_, font)) = parse_font(font_data) {
        Some(font)
    } else {
        None
    }
}

pub fn read_font_file<P: AsRef<Path>>(path: P) -> io::Result<Font> {
    let disp = format!("{}", path.as_ref().display());
    let file = File::open(path)?;
    let mut transcoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(UTF_8))
        .build(file);
    let mut out = String::new();
    transcoded.read_to_string(&mut out)?;
    read_font(&out)
        .ok_or_else(|| Error::new(ErrorKind::Other, format!("Problem with path: {}", disp)))
}

#[test]
fn parse_font_example() {
    let fontstr = include_str!("../fonts/small.flf");
    let res = parse_font(fontstr);
    assert!(res.is_ok());
    if let Ok((_, font)) = res {
        assert!(font.comment.contains("Small by Glenn Chappell"));
        let lowercase_a: Vec<Vec<_>> = [r"       ", r"  __ _ ", r" / _` |", r" \__,_|", r"       "]
            .iter()
            .map(|s| s.chars().collect())
            .collect();
        assert_eq!(font.characters.get(&'a').unwrap().art, lowercase_a);
        let uppercase_a: Vec<Vec<_>> = [
            r"    _   ",
            r"   /_\  ",
            r"  / _ \ ",
            r" /_/ \_\",
            r"        ",
        ]
        .iter()
        .map(|s| s.chars().collect())
        .collect();
        assert_eq!(font.characters.get(&'A').unwrap().art, uppercase_a);
    }
}

#[test]
fn parse_settings_toilet() {
    let settings = "tlf2a 3 3 8 -1 22 0 \r\n";

    assert_eq!(
        terminated(settingsline, line_ending)(settings),
        Ok((
            "",
            Settings {
                hardblank: '',
                charheight: 3,
                baseline: 3,
                maxlength: 8,
                commentlines: 22,
                right2left: false,
                smushmode: SmushMode::empty(),
            }
        ))
    );
}

#[test]
fn parse_settings_standard() {
    // standard.flf
    let settings = r#"flf2a$ 6 5 16 15 11 0 24463 229"#;

    assert_eq!(
        settingsline(settings),
        Ok((
            "",
            Settings {
                hardblank: '$',
                charheight: 6,
                baseline: 5,
                maxlength: 16,
                commentlines: 11,
                right2left: false,
                smushmode: 24463.into(),
            }
        ))
    );
}

#[test]
fn parse_settings_basic() {
    let settings = r#"flf2a$ 8 8 17 -1 2"#;

    assert_eq!(
        settingsline(settings),
        Ok((
            "",
            Settings {
                hardblank: '$',
                charheight: 8,
                baseline: 8,
                maxlength: 17,
                commentlines: 2,
                right2left: false,
                smushmode: SmushMode::empty(),
            }
        ))
    );
}

#[test]
fn parse_settings_broadway() {
    let settings = r#"flf2a$ 11 11 36 2 29"#;

    assert_eq!(
        settingsline(settings),
        Ok((
            "",
            Settings {
                hardblank: '$',
                charheight: 11,
                baseline: 11,
                maxlength: 36,
                commentlines: 29,
                right2left: false,
                smushmode: 130.into(),
            }
        ))
    );
}

#[test]
fn parse_settings_block() {
    let settings = r#"flf2a$ 8 6 27 0 10 0 576 96"#;

    assert_eq!(
        settingsline(settings),
        Ok((
            "",
            Settings {
                hardblank: '$',
                charheight: 8,
                baseline: 6,
                maxlength: 27,
                commentlines: 10,
                right2left: false,
                smushmode: 576.into(),
            }
        ))
    );
}
