use std::{env::args, path::PathBuf};

use elf::{ElfBytes, endian::AnyEndian};
use r68k::{cpu::*, mmu::{MMU}};

pub struct SimpleMMU {
    memory: Vec<u8>,
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

    fn read_word_immutable(&self, addr: u32) -> u16 {
        let hi = self.read_byte_immutable(addr) as u16;
        let lo = self.read_byte_immutable(addr + 1) as u16;

        (hi << 8) | lo
    }

    fn read_long_immutable(&self, addr: u32) -> u32 {
        let hi = self.read_word_immutable(addr) as u32;
        let lo = self.read_word_immutable(addr + 2) as u32;

        (hi << 16) | lo
    }

    fn read_byte(&mut self, addr: u32) -> u8 {
        self.memory[addr as usize & (self.memory.len() - 1)]
    }

    fn read_word(&mut self, addr: u32) -> u16 {
        let hi = self.read_byte(addr) as u16;
        let lo = self.read_byte(addr + 1) as u16;

        (hi << 8) | lo
    }

    fn read_long(&mut self, addr: u32) -> u32 {
        let hi = self.read_word(addr) as u32;
        let lo = self.read_word(addr + 2) as u32;

        (hi << 16) | lo
    }

    fn write_byte(&mut self, addr: u32, val: u8) {
        let addr = addr as usize & (self.memory.len() - 1);
        self.memory[addr] = val;
    }

    fn write_word(&mut self, addr: u32, val: u16) {
        let hi = ((val & 0xff00) >> 8) as u8;
        let lo = (val & 0x00ff) as u8;

        self.write_byte(addr, hi);
        self.write_byte(addr + 1, lo);
    }

    fn write_long(&mut self, addr: u32, val: u32) {
        let hi = ((val & 0xffff0000) >> 16) as u16;
        let lo = (val & 0x0000ffff) as u16;

        self.write_word(addr, hi);
        self.write_word(addr + 2, lo);
    }
}

/// Temporary function to load the data stored at the `.text` section of 
/// an elf32-m68k object file (directly) into the internal memory of the m68k cpu
fn tmp_load_program(obj_file_path_str: &str, cpu: &mut CPU) {
    let path = PathBuf::from(obj_file_path_str);
    let data = std::fs::read(path).unwrap();
    let slice = data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).unwrap();

    write_section_if_exists(cpu, &file, ".vectors");
    write_section_if_exists(cpu, &file, ".text");
    write_section_if_exists(cpu, &file, ".text.startup");   // ?
}

fn write_section_if_exists(cpu: &mut CPU, file: &ElfBytes<'_, AnyEndian>, section_name: &str) {
    let Ok(Some(header)) = file
        .section_header_by_name(section_name) else {
        println!("Failed to parse section header of section '{section_name}'");
        return;
    };

    let addr = header.sh_addr;
    let (data, _) = file
        .section_data(&header)
        .expect("section header does not exist");

    for (i, b) in data.iter().enumerate() {
        cpu.write_mem_byte_internal(addr as u32 + i as u32, *b);
    }
    println!("Wrote section '{}' to address 0x{:08x}", section_name, addr);
}

fn main() {
    let mut mmu = SimpleMMU::new(0x10000000);
    let mut cpu = CPU::new(CPURevision::MC68000, &mut mmu);

    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        println!("need an elf32-m68k object file to load")
    }

    tmp_load_program(&args[1], &mut cpu);

    println!("created cpu");
    cpu.init();
    cpu.print_state();
    loop {
        if !cpu.run() {
            break;
        }
    }
}
