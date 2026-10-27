use crate::{bits::DataSize, cpu::CPURevision};

#[derive(Debug, Clone, Copy)]
pub enum RegisterNum {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

impl From<u8> for RegisterNum {
    fn from(value: u8) -> Self {
        assert!(value <= 7, "Register number must be in the range 0..=7.");

        match value {
            0 => RegisterNum::R0,
            1 => RegisterNum::R1,
            2 => RegisterNum::R2,
            3 => RegisterNum::R3,
            4 => RegisterNum::R4,
            5 => RegisterNum::R5,
            6 => RegisterNum::R6,
            7 => RegisterNum::R7,
            _ => unreachable!(),
        }
    }
}

impl Into<u8> for RegisterNum {
    fn into(self) -> u8 {
        match self {
            RegisterNum::R0 => 0,
            RegisterNum::R1 => 1,
            RegisterNum::R2 => 2,
            RegisterNum::R3 => 3,
            RegisterNum::R4 => 4,
            RegisterNum::R5 => 5,
            RegisterNum::R6 => 6,
            RegisterNum::R7 => 7,
        }
    }
}

pub(crate) struct StatusRegister {
    val: u16,
}

impl StatusRegister {
    fn new() -> StatusRegister {
        StatusRegister {
            val: 0
        }
    }

    pub fn supervisor_state(&self) -> bool {
        self.val & 0b00100000_00000000 != 0
    }

    pub fn set_supervisor(&mut self) {
        self.val |= 0b00100000_00000000
    }

    pub fn sr(&self) -> u16 {
        self.val
    }

    pub fn ccr(&self) -> u8 {
        (self.val & 0xff) as u8
    }

    pub fn modify_sr(&mut self, val: u16, cpu_rev: CPURevision) {
        self.val = val & cpu_rev.sr_flags_mask()
    }

    pub fn modify_ccr(&mut self, val: u16) {
        self.val = (self.val & 0xff00) | (val & 0x1f)
    }

    pub fn x(&self) -> bool {
        self.val & 0b10000 != 0
    }

    pub fn n(&self) -> bool {
        self.val & 0b01000 != 0
    }

    pub fn z(&self) -> bool {
        self.val & 0b00100 != 0
    }

    pub fn v(&self) -> bool {
        self.val & 0b00010 != 0
    }

    pub fn c(&self) -> bool {
        self.val & 0b00001 != 0
    }

    pub fn modify_x(&mut self, cond: bool) {
        if cond { self.set_x() } else { self.clear_x() }
    }

    pub fn modify_n(&mut self, cond: bool) {
        if cond { self.set_n() } else { self.clear_n() }
    }

    pub fn modify_z(&mut self, cond: bool) {
        if cond { self.set_z() } else { self.clear_z() }
    }

    pub fn modify_v(&mut self, cond: bool) {
        if cond { self.set_v() } else { self.clear_v() }
    }

    pub fn modify_c(&mut self, cond: bool) {
        if cond { self.set_c() } else { self.clear_c() }
    }

    pub fn set_x(&mut self) {
        self.val |= 0b10000;
    }

    pub fn set_n(&mut self) {
        self.val |= 0b01000;
    }

    pub fn set_z(&mut self) {
        self.val |= 0b00100;
    }

    pub fn set_v(&mut self) {
        self.val |= 0b00010;
    }

    pub fn set_c(&mut self) {
        self.val |= 0b00001;
    }

    pub fn clear_x(&mut self) {
        self.val &= 0b11111111111_01111;
    }

    pub fn clear_n(&mut self) {
        self.val &= 0b11111111111_10111;
    }

    pub fn clear_z(&mut self) {
        self.val &= 0b11111111111_11011;
    }

    pub fn clear_v(&mut self) {
        self.val &= 0b11111111111_11101;
    }

    pub fn clear_c(&mut self) {
        self.val &= 0b11111111111_11110;
    }
}

pub(crate) struct Registers {
    pub d0: u32,
    pub d1: u32,
    pub d2: u32,
    pub d3: u32,
    pub d4: u32,
    pub d5: u32,
    pub d6: u32,
    pub d7: u32,

    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub a4: u32,
    pub a5: u32,
    pub a6: u32,

    pub usp: u32,
    pub ssp: u32,

    pub pc: u32,

    pub sr: StatusRegister,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            d0: 0,
            d1: 0,
            d2: 0,
            d3: 0,
            d4: 0,
            d5: 0,
            d6: 0,
            d7: 0,

            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,

            usp: 0,
            // TODO: ISP, MSP
            ssp: 0,

            pc: 0,

            sr: StatusRegister::new(),
        }
    }

    /// Read data register dn (d0-d7)
    pub fn read_data(&self, dn: RegisterNum) -> u32 {
        match dn {
            RegisterNum::R0 => self.d0,
            RegisterNum::R1 => self.d1,
            RegisterNum::R2 => self.d2,
            RegisterNum::R3 => self.d3,
            RegisterNum::R4 => self.d4,
            RegisterNum::R5 => self.d5,
            RegisterNum::R6 => self.d6,
            RegisterNum::R7 => self.d7,
        }
    }

    /// Read address register an (a0-a7)
    pub fn read_address(&self, an: RegisterNum) -> u32 {
        match an {
            RegisterNum::R0 => self.a0,
            RegisterNum::R1 => self.a1,
            RegisterNum::R2 => self.a2,
            RegisterNum::R3 => self.a3,
            RegisterNum::R4 => self.a4,
            RegisterNum::R5 => self.a5,
            RegisterNum::R6 => self.a6,
            RegisterNum::R7 => if self.sr.supervisor_state() { self.ssp } else { self.usp },
        }
    }

    /// Write data register dn (d0-d7)
    pub fn write_data(&mut self, dn: RegisterNum, size: DataSize, val: u32) {
        let mask = size.mask();

        match dn {
            RegisterNum::R0 => self.d0 = (self.d0 & !mask) | (val & mask),
            RegisterNum::R1 => self.d1 = (self.d1 & !mask) | (val & mask),
            RegisterNum::R2 => self.d2 = (self.d2 & !mask) | (val & mask),
            RegisterNum::R3 => self.d3 = (self.d3 & !mask) | (val & mask),
            RegisterNum::R4 => self.d4 = (self.d4 & !mask) | (val & mask),
            RegisterNum::R5 => self.d5 = (self.d5 & !mask) | (val & mask),
            RegisterNum::R6 => self.d6 = (self.d6 & !mask) | (val & mask),
            RegisterNum::R7 => self.d7 = (self.d7 & !mask) | (val & mask),
        }
    }

    /// Write a byte value to data register dn (d0-d7) with a given offset
    pub fn write_data_byte_offset(&mut self, dn: RegisterNum, byte_offset: u32, val: u8) {
        let mask = 0xffff_ffffu32 ^ (0xff << (byte_offset * 8));

        match dn {
            RegisterNum::R0 => self.d0 = (self.d0 & mask) | ((val as u32) << (byte_offset * 8)),
            RegisterNum::R1 => self.d1 = (self.d1 & mask) | ((val as u32) << (byte_offset * 8)),
            RegisterNum::R2 => self.d2 = (self.d2 & mask) | ((val as u32) << (byte_offset * 8)),
            RegisterNum::R3 => self.d3 = (self.d3 & mask) | ((val as u32) << (byte_offset * 8)),
            RegisterNum::R4 => self.d4 = (self.d4 & mask) | ((val as u32) << (byte_offset * 8)),
            RegisterNum::R5 => self.d5 = (self.d5 & mask) | ((val as u32) << (byte_offset * 8)),
            RegisterNum::R6 => self.d6 = (self.d6 & mask) | ((val as u32) << (byte_offset * 8)),
            RegisterNum::R7 => self.d7 = (self.d7 & mask) | ((val as u32) << (byte_offset * 8)),
        }
    }

    /// Write address register an (a0-a7)
    pub fn write_address(&mut self, an: RegisterNum, size: DataSize, val: u32) {
        assert_ne!(size, DataSize::Byte);
        let mask = size.mask();

        match an {
            RegisterNum::R0 => self.a0  = (self.a0  & !mask) | (val & mask),
            RegisterNum::R1 => self.a1  = (self.a1  & !mask) | (val & mask),
            RegisterNum::R2 => self.a2  = (self.a2  & !mask) | (val & mask),
            RegisterNum::R3 => self.a3  = (self.a3  & !mask) | (val & mask),
            RegisterNum::R4 => self.a4  = (self.a4  & !mask) | (val & mask),
            RegisterNum::R5 => self.a5  = (self.a5  & !mask) | (val & mask),
            RegisterNum::R6 => self.a6  = (self.a6  & !mask) | (val & mask),
            RegisterNum::R7 => if self.sr.supervisor_state() {
                self.ssp = (self.ssp & !mask) | (val & mask)
            } else {
                self.usp = (self.usp & !mask) | (val & mask)
            },
        }
    }

    /// Increment address register an (a0-a7) by sizeof(byte/word/long)
    pub fn inc_address(&mut self, an: RegisterNum, size: DataSize) {
        match an {
            RegisterNum::R0 => self.a0  += size.sizeof() as u32,
            RegisterNum::R1 => self.a1  += size.sizeof() as u32,
            RegisterNum::R2 => self.a2  += size.sizeof() as u32,
            RegisterNum::R3 => self.a3  += size.sizeof() as u32,
            RegisterNum::R4 => self.a4  += size.sizeof() as u32,
            RegisterNum::R5 => self.a5  += size.sizeof() as u32,
            RegisterNum::R6 => self.a6  += size.sizeof() as u32,
            RegisterNum::R7 => if self.sr.supervisor_state() {
                self.ssp += if let DataSize::Byte = size { 2 } else { size.sizeof() as u32 }
            } else {
                self.usp += if let DataSize::Byte = size { 2 } else { size.sizeof() as u32 }
            },
        }
    }

    /// Decrement address register an (a0-a7) by sizeof(byte/word/long)
    pub fn dec_address(&mut self, an: RegisterNum, size: DataSize) {
        match an {
            RegisterNum::R0 => self.a0  -= size.sizeof() as u32,
            RegisterNum::R1 => self.a1  -= size.sizeof() as u32,
            RegisterNum::R2 => self.a2  -= size.sizeof() as u32,
            RegisterNum::R3 => self.a3  -= size.sizeof() as u32,
            RegisterNum::R4 => self.a4  -= size.sizeof() as u32,
            RegisterNum::R5 => self.a5  -= size.sizeof() as u32,
            RegisterNum::R6 => self.a6  -= size.sizeof() as u32,
            RegisterNum::R7 => if self.sr.supervisor_state() {
                self.ssp -= if let DataSize::Byte = size { 2 } else { size.sizeof() as u32 }
            } else {
                self.usp -= if let DataSize::Byte = size { 2 } else { size.sizeof() as u32 }
            },
        }
    }
}
