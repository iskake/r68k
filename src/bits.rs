#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Size of the given data size in bytes
    /// 
    /// |Size |sizeof |
    /// |-----|-------|
    /// |byte |     1 |
    /// |word |     2 |
    /// |long |     4 |
    pub fn sizeof(self) -> usize {
        match self {
            DataSize::Byte => 1,
            DataSize::Word => 2,
            DataSize::Long => 4,
        }
    }

    /// 32-bit bitmask of the given data size.
    /// 
    /// |Size | Mask         |
    /// |-----|--------------|
    /// |byte | `0xff`       |
    /// |word | `0xffff`     |
    /// |long | `0xffffffff`|
    pub fn mask(self) -> u32 {
        match self {
            DataSize::Byte => 0x0000_00ff,
            DataSize::Word => 0x0000_ffff,
            DataSize::Long => 0xffff_ffff,
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

pub trait SignExtend {
    /// Sign-extend a number of a particular size into a `u32`.
    fn sign_extend(self) -> u32;
}

impl SignExtend for u8 {
    /// Sign-extend a `u8` into a `u32`.
    #[inline(always)]
    fn sign_extend(self) -> u32 {
        self as i8 as i32 as u32
    }
}

impl SignExtend for u16 {
    /// Sign-extend a `u16` into a `u32`.
    #[inline(always)]
    fn sign_extend(self) -> u32 {
        self as i16 as i32 as u32
    }
}
