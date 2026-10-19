use std::{env::args, path::PathBuf};

use elf::{ElfBytes, endian::AnyEndian};
use r68k::cpu::*;

fn main() {
    let mut cpu = CPU::new(CPURevision::M68000);

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
