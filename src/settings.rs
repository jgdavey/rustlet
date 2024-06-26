bitflags! {
    #[repr(transparent)]
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SmushMode: u32 {
        // Horizontal smush rules
        const EQUAL = 1;
        const LOWLINE = 2;
        const HIERARCHY = 4;
        const PAIR = 8;
        const BIGX = 16;
        const HARDBLANK = 32;
        const KERN = 64;
        const SMUSH = 128; // Overrides KERN
        // Vertical smush rules
        const VERT_EQUAL = 256;
        const VERT_LOWLINE = 512;
        const VERT_HIERARCHY = 1024;
        const VERT_PAIR = 2048;
        const VERT_SUPER_SMUSH = 4096;
        const VERT_FIT = 8192;
        const VERT_SMUSH = 16384; // Overrides VERT_FIT
        const OLD_LAYOUT_MASK = Self::EQUAL.bits() | Self::LOWLINE.bits() | Self::HIERARCHY.bits() | Self::PAIR.bits() | Self::BIGX.bits();
    }
}

impl From<u32> for SmushMode {
    fn from(bits: u32) -> Self {
        SmushMode::from_bits_truncate(bits)
    }
}

impl From<SmushMode> for u32 {
    fn from(sm: SmushMode) -> Self {
        sm.bits()
    }
}

impl SmushMode {
    pub fn from_old_layout(bits: i32) -> Self {
        match bits {
            0 => SmushMode::KERN,
            b if b < 0 => SmushMode::empty(),
            _ => {
                let bits = bits as u32;
                (SmushMode::OLD_LAYOUT_MASK & SmushMode::from_bits_truncate(bits))
                    | SmushMode::SMUSH
            }
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Settings {
    pub hardblank: char,
    pub charheight: u32,
    pub baseline: u32,
    pub maxlength: u32,
    pub commentlines: u32,
    pub right2left: bool,
    pub smushmode: SmushMode,
}

impl Settings {
    pub fn is_smush(&self) -> bool {
        self.smushmode.intersects(SmushMode::SMUSH)
    }

    pub fn can_trim_line(&self) -> bool {
        self.smushmode != SmushMode::empty()
    }

    pub fn is_universal_overlap(&self) -> bool {
        !self.smushmode.intersects(SmushMode::from_bits_truncate(63))
    }
}
