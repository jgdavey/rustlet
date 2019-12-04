use crate::settings::{Settings, SmushMode};

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
        return Some(rch)
    }
    if rch == ' ' {
        return Some(lch)
    }

    if !settings.smushmode.intersects(SmushMode::SMUSH) {
        return None
    }

    // Nothing set below 64: this is smushing by universal overlapping
    if !settings.smushmode.intersects(SmushMode::from_bits_truncate(63)) {

        // ensure overlapping preference to visible chars (spaces handled already)
        if lch == settings.hardblank {
            return Some(rch)
        }
        if rch == settings.hardblank {
            return Some(lch)
        }

        // ensure dominant char overlaps, depending on right-to-left parameter
        if settings.right2left {
            return Some(lch)
        }
        return Some(rch)
    }

    if settings.smushmode.intersects(SmushMode::HARDBLANK) {
        if lch == settings.hardblank && rch == settings.hardblank {
            return Some(settings.hardblank)
        }
    } else if lch == settings.hardblank || rch == settings.hardblank {
        return None
    }

    if settings.smushmode.intersects(SmushMode::EQUAL)  {
        if lch == rch {
            return Some(lch)
        }
    }

    if settings.smushmode.intersects(SmushMode::LOWLINE) {
        if lch == '_' && "|/\\[]{}()<>".contains(rch) {
            return Some(rch)
        }
        if rch == '_' && "|/\\[]{}()<>".contains(lch) {
            return Some(lch)
        }
    }

    if settings.smushmode.intersects(SmushMode::HIERARCHY) {
        let hierarchy = vec!["|", "/\\", "[]", "{}", "()", "<>"]; // low -> high precedence
        let is_in_hierarchy = |low: char, high: char, i| {
            let first: &str = hierarchy[i];
            first.contains(low) && (&hierarchy[i+1..]).join("").contains(high)
        };

        for i in 0..hierarchy.len() {
            if is_in_hierarchy(lch, rch, i) {
                return Some(rch)
            }
            if is_in_hierarchy(rch, lch, i) {
                return Some(lch)
            }
        }
    }

    if settings.smushmode.intersects(SmushMode::PAIR) {
        let pairs = vec![['[', ']'], ['{', '}'], ['(', ')']];
        for pair in pairs {
            let [open, close] = pair;
            if (rch == open && lch == close) || (rch == close && lch == open) {
                return Some('|')
            }
        }
    }

    if settings.smushmode.intersects(SmushMode::BIGX) {
        if lch == '/' && rch == '\\' {
            return Some('|')
        }
        if lch == '\\' && rch == '/' {
            return Some('Y')
        }
        if lch == '>' && rch == '<' {
            return Some('X')
        }
    }
    None
}


impl Text {
    fn calculate_smush_amount(&self, other: &Text, settings: &Settings) -> usize {
        let s = settings.smushmode;
        if !s.intersects(SmushMode::SMUSH) && !s.intersects(SmushMode::KERN) {
            return 0
        }

        // For each row of the artwork...
        (0..self.height()).map(|i| {
            let left;
            let right;
            if settings.right2left {
                left = &other.art[i];
                right = &self.art[i];
            } else {
                left = &self.art[i];
                right = &other.art[i];
            }
            let l_blanks = left.iter().rev().take_while(|&c| *c == ' ').count();
            let r_blanks = right.iter().take_while(|&c| *c == ' ').count();
            let mut rowsmush = l_blanks + r_blanks;
            if let Some(&lch) = left.iter().rev().skip_while(|&c| *c == ' ').next() {
                if let Some(&rch) = right.iter().skip_while(|&c| *c == ' ').next() {
                    if let Some(_ch) = smushem(lch, rch, settings) {
                        rowsmush += 1
                    }
                }
            }
            rowsmush
        }).min().unwrap_or(0)
    }

    pub fn append(&mut self, other: &Text, settings: &Settings) -> Result<(), String> {
        let smushamount = self.calculate_smush_amount(other, settings);
        let mut result = if settings.right2left {
            other.art.clone()
        } else {
            self.art.clone()
        };
        for i in 0..self.height() {
            let right = if settings.right2left {
                &self.art[i]
            } else {
                &other.art[i]
            };

            for k in 0..smushamount {
                let column = if smushamount > self.width() {
                    0
                } else {
                    self.width() - smushamount + k
                };
                let rch = right[k];

                if column >= result[i].len() {
                    if rch != ' ' {
                        result[i].push(rch);
                    }
                    continue;
                }
                let lch = result[i][column];

                if let Some(smushed) = smushem(lch, rch, settings) {
                    result[i][column] = smushed;
                }
            }
            result[i].extend_from_slice(&right[smushamount..]);
        }
        self.art = result;
        self.text.push_str(other.text.as_str());
        Ok(())
    }

    pub fn width(&self) -> usize {
        self.art[0].len()
    }

    pub fn height(&self) -> usize {
        self.art.len()
    }

    pub fn empty_of_height(height: u32) -> Self {
        let art: Art = (0..height).map(|_| vec![]).collect();
        Text {
            text: String::from(""),
            art,
        }
    }
}
