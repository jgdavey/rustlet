use bitflags;

bitflags! {
    #[derive(Default)]
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
        const OLD_LAYOUT_MASK = Self::EQUAL.bits | Self::LOWLINE.bits | Self::HIERARCHY.bits | Self::PAIR.bits | Self::BIGX.bits;
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
        if bits < 0 {
            SmushMode::empty()
        } else if bits == 0 {
            SmushMode::KERN
        } else {
            let bits = bits as u32;
            (SmushMode::OLD_LAYOUT_MASK & SmushMode::from_bits_truncate(bits)) | SmushMode::SMUSH
        }
    }
}
