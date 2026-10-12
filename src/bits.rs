pub const SIZE_MASK_LOWER: [u32; 3] = [0x0000_00ff, 0x0000_ffff, 0xffff_ffff];
pub const SIZE_MASK_UPPER: [u32; 3] = [0xffff_ff00, 0xffff_0000, 0x0000_0000];

#[derive(Debug, Clone, Copy)]
pub enum DataSize {
    Byte,
    Word,
    Long,
}

impl std::fmt::Display for DataSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSize::Byte => write!(f, ".b"),
            DataSize::Word => write!(f, ".w"),
            DataSize::Long => write!(f, ".l"),
        }
    }
}

impl DataSize {
    pub fn sizeof(self) -> usize {
        match self {
            DataSize::Byte => 1,
            DataSize::Word => 2,
            DataSize::Long => 4,
        }
    }
}

impl From<u8> for DataSize {
    fn from(value: u8) -> Self {
        assert!(value <= 2, "Data size must be one of 0, 1, or 2.");

        match value {
            0 => DataSize::Byte,
            1 => DataSize::Word,
            2 => DataSize::Long,
            _ => unreachable!(),
        }
    }
}

impl Into<u8> for DataSize {
    fn into(self) -> u8 {
        match self {
            DataSize::Byte => 0,
            DataSize::Word => 1,
            DataSize::Long => 2,
        }
    }
}

impl Into<usize> for DataSize {
    fn into(self) -> usize {
        match self {
            DataSize::Byte  => 0,
            DataSize::Word => 1,
            DataSize::Long  => 2,
        }
    }
}

pub fn is_zero(val: u32, size: DataSize) -> bool {
    // Assuming size: 0 is byte and size: 2 is long

    // Shift amounts:
    // - b: 8*3 = 24
    // - s: 8*2 = 16
    // - l: 8*0 = 0
    let size: usize = size.into();
    let s = 1 << size;
    let shift = ((s & 0b11) | ((s & 1) << 1)) * 8;
    val & (0xffff_ffff >> shift) == 0
}

pub fn is_negative(val: u32, size: DataSize) -> bool {
    // Assuming size: 0 is byte and size: 2 is long

    // We want to mask the top bit of either byte no. 0 (byte), 1 (short), or 4 (long)
    // Tested bit (hex):
    // - b: xxxx_xx8x
    // - s: xxxx_8xxx
    // - l: 8xxx_xxxx
    let size: usize = size.into();
    let shift = ((1 << size) & !0b1) * 8;
    let mask = 0x80 << shift;

    val & mask != 0
}

/// Sign-extend a byte to a long
pub fn sign_extend_byte(val: u8) -> u32 {
    let v = val as u32;
    (0xffff_ff00 << ((!v & 0x80) >> 7) * 16) | (v & 0xff)
}

/// Sign extend a byte to a word or a word to a long
pub fn sign_extend(to_size: DataSize, val: u32) -> u32 {
    match to_size {
        DataSize::Byte => panic!("Cannot sign-extend with .b"),
        DataSize::Word => ((0xff00 << ((!val & 0x80) >> 7) * 8) | (val & 0xff)) & 0xffff,
        DataSize::Long => (0xffff_0000 << ((!val & 0x8000) >> 15) * 16) | (val & 0xffff),
    }
}