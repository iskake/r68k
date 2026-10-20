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
    /// Size of the given data size in bytes.
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

    /// 32-bit bitmask of the signed representation given data size.
    /// 
    /// |Size | Mask         |
    /// |-----|--------------|
    /// |byte | `0x7f`       |
    /// |word | `0x7fff`     |
    /// |long | `0x7fffffff`|
    pub fn signed_mask(self) -> u32 {
        match self {
            DataSize::Byte => 0x0000_007f,
            DataSize::Word => 0x0000_7fff,
            DataSize::Long => 0x7fff_ffff,
        }
    }

    /// Sign bit of the given data size.
    /// 
    /// |Size | Mask         |
    /// |-----|--------------|
    /// |byte | `0x80`       |
    /// |word | `0x8000`     |
    /// |long | `0x80000000`|
    pub fn sign_bit(self) -> u32 {
        match self {
            DataSize::Byte => 0x0000_0080,
            DataSize::Word => 0x0000_8000,
            DataSize::Long => 0x8000_0000,
        }
    }
}

impl From<u8> for DataSize {
    fn from(value: u8) -> Self {
        match value {
            0 => DataSize::Byte,
            1 => DataSize::Word,
            2 => DataSize::Long,
            _ => panic!("Data size must be one of 0, 1, or 2"),
        }
    }
}

impl Into<u8> for DataSize {
    fn into(self) -> u8 {
        self as u8
    }
}

impl Into<usize> for DataSize {
    fn into(self) -> usize {
        self as usize
    }
}

pub fn is_zero(val: u32, size: DataSize) -> bool {
    (val & size.mask()) == 0
}

pub fn is_negative(val: u32, size: DataSize) -> bool {
    (val & size.sign_bit()) != 0
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
