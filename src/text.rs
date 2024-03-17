use crate::font::Font;
use crate::settings::{Settings, SmushMode};
use std::cmp::min;
use std::fmt;

type Art = Vec<Vec<char>>;

#[derive(Debug, PartialEq, Clone)]
pub struct Text {
    /// the artwork, as lines
    pub art: Art,
    /// the unadorned text
    pub text: String,
}

/// Given 2 characters, attempts to smush them into 1, according to
/// smushmode.  Returns smushed character or '\0' if no smushing can be
/// done.

// smushmode values are sum of following (all values smush blanks):
// 1: Smush equal chars (not hardblanks)
// 2: Smush '_' with any char in hierarchy below
// 4: hierarchy: "|", "/\", "[]", "{}", "()", "<>"
//    Each class in hier. can be replaced by later class.
// 8: [ + ] -> |, { + } -> |, ( + ) -> |
// 16: / + \ -> X, > + < -> X (only in that order)
// 32: hardblank + hardblank -> hardblank
fn smushem(lch: char, rch: char, settings: &Settings) -> Option<char> {
    if lch == ' ' {
        return Some(rch);
    }
    if rch == ' ' {
        return Some(lch);
    }

    if !settings.smushmode.intersects(SmushMode::SMUSH) {
        return None;
    }

    // Nothing set below 64: this is smushing by universal overlapping
    if !settings
        .smushmode
        .intersects(SmushMode::from_bits_truncate(63))
    {
        // ensure overlapping preference to visible chars (spaces handled already)
        if lch == settings.hardblank {
            return Some(rch);
        }
        if rch == settings.hardblank {
            return Some(lch);
        }

        // ensure dominant char overlaps, depending on right-to-left parameter
        if settings.right2left {
            return Some(lch);
        }
        return Some(rch);
    }

    if settings.smushmode.intersects(SmushMode::HARDBLANK) {
        if lch == settings.hardblank && rch == settings.hardblank {
            return Some(settings.hardblank);
        }
    } else if lch == settings.hardblank || rch == settings.hardblank {
        return None;
    }

    if settings.smushmode.intersects(SmushMode::EQUAL) && lch == rch {
        return Some(lch);
    }

    if settings.smushmode.intersects(SmushMode::LOWLINE) {
        if lch == '_' && "|/\\[]{}()<>".contains(rch) {
            return Some(rch);
        }
        if rch == '_' && "|/\\[]{}()<>".contains(lch) {
            return Some(lch);
        }
    }

    if settings.smushmode.intersects(SmushMode::HIERARCHY) {
        let hierarchy = ["|", "/\\", "[]", "{}", "()", "<>"]; // low -> high precedence
        let is_in_hierarchy = |low: char, high: char, i| {
            let first: &str = hierarchy[i];
            first.contains(low) && hierarchy[i + 1..].join("").contains(high)
        };

        for i in 0..hierarchy.len() {
            if is_in_hierarchy(lch, rch, i) {
                return Some(rch);
            }
            if is_in_hierarchy(rch, lch, i) {
                return Some(lch);
            }
        }
    }

    if settings.smushmode.intersects(SmushMode::PAIR) {
        let pairs = vec![['[', ']'], ['{', '}'], ['(', ')']];
        for pair in pairs {
            let [open, close] = pair;
            if (rch == open && lch == close) || (rch == close && lch == open) {
                return Some('|');
            }
        }
    }

    if settings.smushmode.intersects(SmushMode::BIGX) {
        if lch == '/' && rch == '\\' {
            return Some('|');
        }
        if lch == '\\' && rch == '/' {
            return Some('Y');
        }
        if lch == '>' && rch == '<' {
            return Some('X');
        }
    }
    None
}

impl Text {
    fn calculate_smush_amount(&self, other: &Text, settings: &Settings) -> usize {
        let s = settings.smushmode;
        if !s.intersects(SmushMode::SMUSH) && !s.intersects(SmushMode::KERN) {
            return 0;
        }
        if settings.right2left {
            if other.width() == 0 {
                return 0;
            }
        } else {
            if self.width() == 0 {
                return 0;
            }
        }

        // For each row of the artwork...
        let answer = (0..self.height())
            .map(|row| {
                let (left, right, ch1, ch2) = if settings.right2left {
                    (&other.art[row], &self.art[row], other.art[row].first(), self.art[row].last())
                } else {
                    (&self.art[row], &other.art[row], self.art[row].last(), other.art[row].first())
                };
                let l_blanks = left.iter().rev().skip(1).take_while(|&c| *c == ' ').count();
                let r_blanks = right.iter().skip(1).take_while(|&c| *c == ' ').count();
                let mut rowsmush = l_blanks + r_blanks;
                match (ch1, ch2) {
                    (None, _) | (Some(' '), _) => rowsmush += 1,
                    (Some(&c1), Some(&c2)) => {
                        if let Some(_) = smushem(c1, c2, settings) {
                            rowsmush += 1
                        }
                    }
                    _ => ()
                }
                rowsmush
            })
            .min()
            .unwrap_or(0);
        // println!("rowsmuch: {}, text: {}", answer, self.text);
        answer
    }

    pub fn append(&self, other: &Text, settings: &Settings) -> Text {
        // println!("append {} <- {}", self.text, other.text);
        let smushamount = self.calculate_smush_amount(other, settings);
        let left;
        let right;
        if settings.right2left {
            if self.width() == 0 {
                return other.clone();
            }
            left = other;
            right = self;
        } else {
            left = self;
            right = other;
        }

        let mut result = left.art.clone();
        let mut text = left.text.clone();
        text.push_str(right.text.as_str());

        for (i, item) in result.iter_mut().enumerate().take(self.height()) {
            for k in 0..smushamount {
                let column = self.width() + k - smushamount;
                let rch = right.art[i][k];

                if column >= item.len() {
                    if rch != ' ' {
                        item.push(rch);
                    }
                    continue;
                }
                let lch = item[column];

                if let Some(smushed) = smushem(lch, rch, settings) {
                    item[column] = smushed;
                }
            }
            let x = min(right.width(), smushamount);
            item.extend_from_slice(&right.art[i][x..])
        }
        Text { art: result, text }
    }

    pub fn width(&self) -> usize {
        self.art[0].len()
    }

    pub fn height(&self) -> usize {
        self.art.len()
    }

    pub fn is_empty(&self) -> bool {
        self.art.is_empty() || self.art[0].is_empty()
    }

    pub fn empty_of_height(height: u32) -> Self {
        let art: Art = (0..height).map(|_| vec![]).collect();
        Text {
            text: String::from(""),
            art,
        }
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.art {
            for &ch in line.iter() {
                write!(f, "{}", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Given a message, font, settings, and maximum width, formats the
/// message using the font. Returns "art lines", that is, a
/// Vec<String> that, when printed sequentially, will resemble "lines"
/// of text. Each String can be multiple lines.
pub fn art_lines(message: &str, font: &Font, settings: &Settings, max_width: usize) -> Vec<String> {
    let space = font.get_character(&' ');
    let words: Vec<Text> = message
        .split_whitespace()
        .flat_map(|word| {
            let mut result = vec![];
            let mut line = Text::empty_of_height(font.height());
            for ch in word.chars().map(|c| font.get_character(&c)) {
                let new_line = line.append(ch, settings);
                if !line.is_empty() && new_line.width() > max_width {
                    result.push(line);
                    line = Text::empty_of_height(font.height()).append(ch, settings);
                } else {
                    line = new_line
                }
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
        if line.is_empty() {
            line = line.append(&word, settings);
        } else {
            let new_line = line.append(space, settings).append(&word, settings);
            if new_line.width() > max_width {
                result.push(line);
                line = Text::empty_of_height(font.height()).append(&word, settings);
            } else {
                line = new_line;
            }
        }
    }

    if !line.is_empty() {
        result.push(line);
    }

    result
        .iter()
        .map(|art_line| art_line.to_string().replace(font.hardblank(), " "))
        .collect()
}
