// AD-3: on-disk header flag byte.
// Bit layout (LSB first): empty(0) lower_inc(1) upper_inc(2) lower_inf(3)
// upper_inf(4) canonical(5) reserved(6,7).
// Infinities and empty are flags, never subtype sentinels (SEED key decision 4).

pub const FLAG_EMPTY: u8 = 1 << 0;
pub const FLAG_LOWER_INC: u8 = 1 << 1;
pub const FLAG_UPPER_INC: u8 = 1 << 2;
pub const FLAG_LOWER_INF: u8 = 1 << 3;
pub const FLAG_UPPER_INF: u8 = 1 << 4;
pub const FLAG_CANONICAL: u8 = 1 << 5;

pub const VERSION_NIBBLE: u8 = 0x40; // version 1 in bits 6-7 (high 2 bits)

pub const HEADER_LEN: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub version: u8,
    pub empty: bool,
    pub lower_inc: bool,
    pub upper_inc: bool,
    pub lower_inf: bool,
    pub upper_inf: bool,
    pub canonical: bool,
}

impl Header {
    pub fn decode(byte: u8) -> Self {
        Header {
            version: (byte & 0xC0) >> 6,
            empty: byte & FLAG_EMPTY != 0,
            lower_inc: byte & FLAG_LOWER_INC != 0,
            upper_inc: byte & FLAG_UPPER_INC != 0,
            lower_inf: byte & FLAG_LOWER_INF != 0,
            upper_inf: byte & FLAG_UPPER_INF != 0,
            canonical: byte & FLAG_CANONICAL != 0,
        }
    }

    pub fn encode(&self) -> u8 {
        let mut b = (self.version & 0x03) << 6;
        if self.empty {
            b |= FLAG_EMPTY;
        }
        if self.lower_inc {
            b |= FLAG_LOWER_INC;
        }
        if self.upper_inc {
            b |= FLAG_UPPER_INC;
        }
        if self.lower_inf {
            b |= FLAG_LOWER_INF;
        }
        if self.upper_inf {
            b |= FLAG_UPPER_INF;
        }
        if self.canonical {
            b |= FLAG_CANONICAL;
        }
        b
    }
}
