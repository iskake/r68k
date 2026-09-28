use r68k::cpu::*;

fn main() {
    let mut cpu = CPU::new(CPURevision::M68000);

    println!("created cpu");
    cpu.run()
}