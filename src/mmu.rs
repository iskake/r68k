extern crate alloc;
use alloc::vec;

pub trait MMU {
    fn read_byte_immutable(&self, addr: u32) -> u8;
    fn read_short_immutable(&self, addr: u32) -> u16;
    fn read_long_immutable(&self, addr: u32) -> u32;

    fn read_byte(&mut self, addr: u32) -> u8;
    fn read_short(&mut self, addr: u32) -> u16;
    fn read_long(&mut self, addr: u32) -> u32;

    fn write_byte(&mut self, addr: u32, val: u8);
    fn write_short(&mut self, addr: u32, val: u16);
    fn write_long(&mut self, addr: u32, val: u32);
}

pub struct SimpleMMU {
    memory: alloc::vec::Vec<u8>,
}

impl SimpleMMU {
    pub fn new(size: usize) -> SimpleMMU {
        assert!(size.count_ones() == 1, "Memory size must be a multiple of 2");

        SimpleMMU {
            memory: vec![0; size],
        }
    }
}

impl MMU for SimpleMMU {
    fn read_byte_immutable(&self, addr: u32) -> u8 {
        self.memory[addr as usize & (self.memory.len() - 1)]
    }

    fn read_short_immutable(&self, addr: u32) -> u16 {
        let hi = self.read_byte_immutable(addr) as u16;
        let lo = self.read_byte_immutable(addr + 1) as u16;

        (hi << 8) | lo
    }

    fn read_long_immutable(&self, addr: u32) -> u32 {
        let hi = self.read_short_immutable(addr) as u32;
        let lo = self.read_short_immutable(addr + 2) as u32;

        (hi << 16) | lo
    }

    fn read_byte(&mut self, addr: u32) -> u8 {
        self.memory[addr as usize & (self.memory.len() - 1)]
    }

    fn read_short(&mut self, addr: u32) -> u16 {
        let hi = self.read_byte(addr) as u16;
        let lo = self.read_byte(addr + 1) as u16;

        (hi << 8) | lo
    }

    fn read_long(&mut self, addr: u32) -> u32 {
        let hi = self.read_short(addr) as u32;
        let lo = self.read_short(addr + 2) as u32;

        (hi << 16) | lo
    }

    fn write_byte(&mut self, addr: u32, val: u8) {
        let addr = addr as usize & (self.memory.len() - 1);
        self.memory[addr] = val;
    }

    fn write_short(&mut self, addr: u32, val: u16) {
        let hi = ((val & 0xff00) >> 8) as u8;
        let lo = (val & 0x00ff) as u8;

        self.write_byte(addr, hi);
        self.write_byte(addr + 1, lo);
    }

    fn write_long(&mut self, addr: u32, val: u32) {
        let hi = ((val & 0xffff0000) >> 16) as u16;
        let lo = (val & 0x0000ffff) as u16;

        self.write_short(addr, hi);
        self.write_short(addr + 2, lo);
    }
}
