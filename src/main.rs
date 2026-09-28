use r68k::cpu::*;

fn main() {
    let mut cpu = CPU::new(CPURevision::M68000);

    println!("created cpu");
    cpu.run();
    cpu.dbg_run_op(0b0010000000000000);
}