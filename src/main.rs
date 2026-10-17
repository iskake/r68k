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
    cpu.print_state();
    loop {
        cpu.run();
    }
}

/// Temporary function to load the data stored at the `.text` section of 
/// an elf32-m68k object file (directly) into the internal memory of the m68k cpu
fn tmp_load_program(obj_file_path_str: &str, cpu: &mut CPU) {
    let path = PathBuf::from(obj_file_path_str);
    let data = std::fs::read(path).unwrap();
    let slice = data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).unwrap();

    let text_header = file
        .section_header_by_name(".text")
        .expect("section table header could not be parsed")
        .expect("no .text header found");

    let (text, _) = file
        .section_data(&text_header)
        .expect("section header does not exist");

    for (i, b) in text.iter().enumerate() {
        cpu.write_mem_byte_internal(i as u32, *b);
    }
}
