use crate::{cpu::inst::AddressingMode, mmu::MMU};

mod inst;

struct StatusRegister {
    val: u16,
}

impl StatusRegister {
    fn new() -> StatusRegister {
        StatusRegister {
            val: 0
        }
    }

    fn hi(&self) -> u8 {
        ((self.val & 0xff00) >> 8) as u8
    }

    fn ccr(&self) -> u8 {
        (self.val & 0xff) as u8
    }

    fn modify_x(&mut self, cond: bool) {
        if cond { self.set_x() } else { self.clear_x() }
    }

    fn modify_n(&mut self, cond: bool) {
        if cond { self.set_n() } else { self.clear_n() }
    }

    fn modify_z(&mut self, cond: bool) {
        if cond { self.set_z() } else { self.clear_z() }
    }

    fn modify_v(&mut self, cond: bool) {
        if cond { self.set_v() } else { self.clear_v() }
    }

    fn modify_c(&mut self, cond: bool) {
        if cond { self.set_c() } else { self.clear_c() }
    }

    fn set_x(&mut self) {
        self.val |= 0b10000;
    }

    fn set_n(&mut self) {
        self.val |= 0b01000;
    }

    fn set_z(&mut self) {
        self.val |= 0b00100;
    }

    fn set_v(&mut self) {
        self.val |= 0b00010;
    }

    fn set_c(&mut self) {
        self.val |= 0b00001;
    }

    fn clear_x(&mut self) {
        self.val &= 0b11111111111_01111;
    }

    fn clear_n(&mut self) {
        self.val &= 0b11111111111_10111;
    }

    fn clear_z(&mut self) {
        self.val &= 0b11111111111_11011;
    }

    fn clear_v(&mut self) {
        self.val &= 0b11111111111_11101;
    }

    fn clear_c(&mut self) {
        self.val &= 0b11111111111_11110;
    }
}

struct Registers {
    d0: u32,
    d1: u32,
    d2: u32,
    d3: u32,
    d4: u32,
    d5: u32,
    d6: u32,
    d7: u32,

    a0: u32,
    a1: u32,
    a2: u32,
    a3: u32,
    a4: u32,
    a5: u32,
    a6: u32,

    usp: u32,
    ssp: u32,

    pc: u32,

    sr: StatusRegister,
}

impl Registers {
    fn new() -> Registers {
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
            ssp: 0,

            pc: 0,

            sr: StatusRegister::new(),
        }
    }
}

pub enum CPURevision {
    // Temp.
    M68000,
    M68010,
    M68020,
    M68030,
    M68040,
    M68060,
}

pub struct CPU {
    reg: Registers,

    mmu: MMU,

    cycles: usize,
    revision: CPURevision,
}

impl CPU {
    pub fn new(rev: CPURevision) -> CPU {
        match rev {
            CPURevision::M68000 => {},
            CPURevision::M68010 => todo!("68010 support"),
            CPURevision::M68020 => todo!("68020 support"),
            CPURevision::M68030 => todo!("68030 support"),
            CPURevision::M68040 => todo!("68040 support"),
            CPURevision::M68060 => todo!("68060 support"),
        }

        CPU {
            reg: Registers::new(),
            mmu: MMU::new(0x10000),
            cycles: 0,
            revision: rev,
        }
    }

    pub fn cpu_name(&self) -> String {
        String::from("")
    }

    pub fn print_state(&self) {
        println!("CPU STATE");
        println!(
            "  d0: {:08x}, d1: {:08x}, d2: {:08x}, d3: {:08x}, d4: {:08x}, d5: {:08x}, d6: {:08x}, d7: {:08x}",
            self.reg.d0,
            self.reg.d1,
            self.reg.d2,
            self.reg.d3,
            self.reg.d4,
            self.reg.d5,
            self.reg.d6,
            self.reg.d7,
        );
        println!(
            "  a0: {:08x}, a1: {:08x}, a2: {:08x}, a3: {:08x}, a4: {:08x}, a5: {:08x}, a6: {:08x}, sp: {:08x} ({:08x})",
            self.reg.a0,
            self.reg.a1,
            self.reg.a2,
            self.reg.a3,
            self.reg.a4,
            self.reg.a5,
            self.reg.a6,
            self.reg.usp,
            self.reg.ssp,
        );
        println!("  sr hi: {:08b} ccr: {:08b}", self.reg.sr.hi(), self.reg.sr.ccr());
        println!("  pc: {:08x}", self.reg.pc);
    }

    fn read_am(&mut self, size: u8, am: AddressingMode) -> u32 {
        // Should read a byte/short/long (based on size) from the location specified by the addressing mode
        todo!()
    }

    fn read_pc_inc_short(&mut self) -> u16 {
        let val = self.mmu.read_short(self.reg.pc);
        self.reg.pc += 2;
        val
    }

    fn fetch_and_run(&mut self) {
        let op = self.read_pc_inc_short();
        let inst = inst::decode_opcode(op);
        println!("Got instruction: {inst:?}");
        inst.run(self);
    }

    pub fn run(&mut self) {
        self.print_state();
        self.fetch_and_run();
    }

    pub fn dbg_run_op(&mut self, op: u16) {
    }
}
