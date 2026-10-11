pub fn is_zero(val: u32, size: u8) -> bool {
    // Assuming size: 0 is byte and size: 2 is long

    // Shift amounts:
    // - b: 8*3 = 24
    // - s: 8*2 = 16
    // - l: 8*0 = 0
    let s = 1 << size;
    let shift = ((s & 0b11) | ((s & 1) << 1)) * 8;
    val & (0xffff_ffff >> shift) == 0
}

pub fn is_negative(val: u32, size: u8) -> bool {
    // Assuming size: 0 is byte and size: 2 is long

    // We want to mask the top bit of either byte no. 0 (byte), 1 (short), or 4 (long)
    // Tested bit (hex):
    // - b: xxxx_xx8x
    // - s: xxxx_8xxx
    // - l: 8xxx_xxxx
    let shift = ((1 << size) & !0b1) * 8;
    let mask = 0x80 << shift;

    val & mask != 0
}