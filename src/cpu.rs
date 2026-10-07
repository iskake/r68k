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

    fn fetch_and_run(&mut self) {

    }

    pub fn run(&mut self) {
        self.print_state();
    }

    pub fn dbg_run_op(&mut self, op: u16) {
        let inst = inst::decode(op);

        println!("Got instruction: {inst:?}")
    }
}
