use r68k::cpu::*;

fn main() {
    let mut cpu = CPU::new(CPURevision::M68000);

    // Temp. test code just to check that it works at all...!
    cpu.write_mem_short_internal(0x0000_0000, 0b0100_011_010_000_000);  // not.l d0 (d0 -> ffffffff)
    cpu.write_mem_short_internal(0x0000_0002, 0b0010_000_001_111_100);  // movea.l a0,#00001001
    cpu.write_mem_long_internal(0x0000_0004, 0x0000_1001);
    cpu.write_mem_short_internal(0x0000_0008, 0b0010_001_001_100_000);  // movea.l a1,-(a0)
    cpu.write_mem_long_internal(0x0000_0ffd, 0x0000_0002);
    cpu.write_mem_short_internal(0x0000_000a, 0b0010_011_001_101_001);  // movea.l a3,2(a1)
    cpu.write_mem_short_internal(0x0000_000c, 0x0000_0002);
    cpu.write_mem_short_internal(0x0000_000e, 0b0011_011_001_101_001);  // movea.w a3,0(a1)
    cpu.write_mem_short_internal(0x0000_0012, 0b0000_000_101_001_001);  // movep.l (0,a1),d0
    // cpu.write_mem_short_internal(0x0000_000a, 0b0100_011_001_001_001);

    println!("created cpu");
    cpu.print_state();
    loop {
        cpu.run();
    }
}