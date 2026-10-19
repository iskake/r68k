use crate::{bits::{DataSize, SIZE_MASK_LOWER, SIZE_MASK_UPPER, sign_extend, sign_extend_byte}, cpu::inst::AddressingMode, mmu::MMU};

mod inst;

const CPU_VECTOR_SIZE: u32 = 4;

pub enum CPUVector {
    ResetSP            = 0,
    ResetPC            = 1,
    AccessFault        = 2,
    AddressError       = 3,

    IllegalInstruction = 4,
    IntDivByZero       = 5,
    ChkInstruction     = 6,
    TRAPV              = 7,

    PrivelegeViolation = 8,
    TRACE              = 9,
    Line1010Emulator   = 10,
    Line1111Emulator   = 11,

    // [Unassigned #12]
    CoprocessorProtocolViolation = 13,
    FormatError            = 14,
    UninitializedInterrupt = 15,

    // [Unassigned #16-#23]

    SpuriousInterrupt  = 24,
    InterruptAutovectorLevel1 = 25,
    InterruptAutovectorLevel2 = 26,
    InterruptAutovectorLevel3 = 27,
    InterruptAutovectorLevel4 = 28,
    InterruptAutovectorLevel5 = 29,
    InterruptAutovectorLevel6 = 30,
    InterruptAutovectorLevel7 = 31,

    // [TRAP 0-15 (#32-#47)]

    FPBranchOrSet = 48,
    FPInexactResult = 49,
    FPDivByDero = 50,
    FPUnderflow = 51,

    FPOperandError = 52,
    FPOverflow = 53,
    FPSignalingNaN = 54,
    FPUnimplementedDataType = 55,

    MMUConfigError = 56,
    MMUIllegalOpError = 57,
    MMUAccessLevelViolationError = 58,

    // [Unassigned #59-#63]

    // [User defined #64-#255]
}


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

struct StatusRegister {
    val: u16,
}

impl StatusRegister {
    fn new() -> StatusRegister {
        StatusRegister {
            val: 0
        }
    }

    fn supervisor_state(&self) -> bool {
        self.val & 0b00100000_00000000 != 0
    }

    fn set_supervisor(&mut self) {
        self.val |= 0b00100000_00000000
    }

    fn clear_supervisor(&mut self) {
        self.val &= 0b11011111_11111111
    }

    fn sr(&self) -> u16 {
        self.val
    }

    fn sr_hi(&self) -> u8 {
        ((self.val & 0xff00) >> 8) as u8
    }

    fn ccr(&self) -> u8 {
        (self.val & 0xff) as u8
    }

    fn modify_sr(&mut self, val: u16) {
        self.val = val
    }

    fn modify_ccr(&mut self, val: u16) {
        self.val = (self.val & 0xff00) | (val & 0xff)
    }

    fn x(&self) -> bool {
        self.val & 0b10000 != 0
    }

    fn n(&self) -> bool {
        self.val & 0b01000 != 0
    }

    fn z(&self) -> bool {
        self.val & 0b00100 != 0
    }

    fn v(&self) -> bool {
        self.val & 0b00010 != 0
    }

    fn c(&self) -> bool {
        self.val & 0b00001 != 0
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

    /// Read data register dn (d0-d7)
    fn read_data(&self, dn: RegisterNum) -> u32 {
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
    fn read_address(&self, an: RegisterNum) -> u32 {
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
    fn write_data(&mut self, dn: RegisterNum, size: DataSize, val: u32) {
        let s: usize = size.into();

        match dn {
            RegisterNum::R0 => self.d0 = (self.d0 & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R1 => self.d1 = (self.d1 & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R2 => self.d2 = (self.d2 & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R3 => self.d3 = (self.d3 & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R4 => self.d4 = (self.d4 & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R5 => self.d5 = (self.d5 & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R6 => self.d6 = (self.d6 & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R7 => self.d7 = (self.d7 & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
        }
    }

    /// Write a byte value to data register dn (d0-d7) with a given offset
    fn write_data_byte_offset(&mut self, dn: RegisterNum, byte_offset: u32, val: u8) {
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
    fn write_address(&mut self, an: RegisterNum, size: DataSize, val: u32) {
        let s: usize = size.into();

        match an {
            RegisterNum::R0 => self.a0  = (self.a0  & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R1 => self.a1  = (self.a1  & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R2 => self.a2  = (self.a2  & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R3 => self.a3  = (self.a3  & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R4 => self.a4  = (self.a4  & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R5 => self.a5  = (self.a5  & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R6 => self.a6  = (self.a6  & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s]),
            RegisterNum::R7 => if self.sr.supervisor_state() {
                self.ssp = (self.ssp & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s])
            } else {
                self.usp = (self.usp & SIZE_MASK_UPPER[s]) | (val & SIZE_MASK_LOWER[s])
            },
        }
    }

    /// Increment address register an (a0-a7) by sizeof(byte/word/long)
    fn inc_address(&mut self, an: RegisterNum, size: DataSize) {
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
    fn dec_address(&mut self, an: RegisterNum, size: DataSize) {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    data_bus: u32,
    addr_bus: u32,

    stopped: bool,
    reset_signal: bool,

    cycles: usize,
    read_cycles: usize,
    write_cycles: usize,

    pub revision: CPURevision,
    // Avoids having to match based on revision
    data_bus_mask: u32,
    addr_bus_mask: u32,
}

impl CPU {
    pub fn new(rev: CPURevision) -> CPU {
        let (data_bus_mask, addr_bus_mask) = match rev {
            CPURevision::M68000 => { (0x0000_ffff, 0x00ff_ffff) },
            CPURevision::M68010 => todo!("68010 support"),
            CPURevision::M68020 => todo!("68020 support"),
            CPURevision::M68030 => todo!("68030 support"),
            CPURevision::M68040 => todo!("68040 support"),
            CPURevision::M68060 => todo!("68060 support"),
        };

        CPU {
            reg: Registers::new(),
            mmu: MMU::new(0x1000000),

            data_bus: 0,
            addr_bus: 0,

            stopped: false,
            reset_signal: false,

            cycles: 0,
            read_cycles: 0,
            write_cycles: 0,

            revision: rev,
            data_bus_mask,
            addr_bus_mask,
        }
    }

    pub fn cpu_name(&self) -> String {
        match self.revision {
            CPURevision::M68000 => String::from("MC68000"),
            CPURevision::M68010 => String::from("MC68010"),
            CPURevision::M68020 => String::from("MC68020"),
            CPURevision::M68030 => String::from("MC68030"),
            CPURevision::M68040 => String::from("MC68040"),
            CPURevision::M68060 => String::from("MC68060"),
        }
    }

    pub fn init(&mut self) {
        self.reg.sr.set_supervisor();
        self.reg.ssp = self.read_mem_long(CPUVector::ResetSP as u32 * CPU_VECTOR_SIZE);
        self.reg.pc = self.read_mem_long(CPUVector::ResetPC as u32 * CPU_VECTOR_SIZE);
    }

    pub fn print_state(&self) {
        println!("CPU STATE ({})", self.cpu_name());
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
            "  a0: {:08x}, a1: {:08x}, a2: {:08x}, a3: {:08x}, a4: {:08x}, a5: {:08x}, a6: {:08x}, a7: {:08x} ({})",
            self.reg.a0,
            self.reg.a1,
            self.reg.a2,
            self.reg.a3,
            self.reg.a4,
            self.reg.a5,
            self.reg.a6,
            self.reg.read_address(RegisterNum::R7),
            if self.reg.sr.supervisor_state() { "ssp" } else { "usp" }
        );
        println!(
            "  sr: {:08b}_{:08b}                                                                           ({}: {:08x})",
            self.reg.sr.sr_hi(),
            self.reg.sr.ccr(),
            if self.reg.sr.supervisor_state() { "usp" } else { "ssp" },
            if self.reg.sr.supervisor_state() { self.reg.usp } else { self.reg.ssp },
        );
        println!(
            "  pc: {:08x}, cycles: {}({}/{}), data bus: {:08x}, addr bus: {:08x}",
            self.reg.pc,
            self.cycles,
            self.read_cycles,
            self.write_cycles,
            self.data_bus,
            self.addr_bus
        );
    }

    fn update_data_bus(&mut self, data: u32) {
        self.data_bus = data & self.data_bus_mask;
    }

    fn update_addr_bus(&mut self, addr: u32) {
        self.addr_bus = addr & self.addr_bus_mask;
    }

    /// Cycles: 0(0/0)
    pub fn write_mem_byte_internal(&mut self, addr: u32, val: u8) {
        self.mmu.write_byte(addr & self.addr_bus_mask, val);
    }

    /// Cycles: 0(0/0)
    pub fn write_mem_short_internal(&mut self, addr: u32, val: u16) {
        self.mmu.write_short(addr & self.addr_bus_mask, val);
    }

    /// Cycles: 0(0/0)
    pub fn write_mem_long_internal(&mut self, addr: u32, val: u32) {
        self.mmu.write_long(addr & self.addr_bus_mask, val);
    }

    /// Cycles: 4(0/1)
    fn write_mem_byte(&mut self, addr: u32, val: u8) {
        self.update_data_bus(val as u32);
        self.update_addr_bus(addr);

        self.write_mem_byte_internal(self.addr_bus, val);
        self.cycle_write();
    }

    /// Cycles: 4(0/1)
    fn write_mem_short(&mut self, addr: u32, val: u16) {
        self.update_data_bus(val as u32);
        self.update_addr_bus(addr);

        self.write_mem_short_internal(addr, val);
        self.cycle_write();
    }

    /// Cycles: 8(0/2)
    fn write_mem_long(&mut self, addr: u32, val: u32) {
        if self.data_bus_mask & 0x8000_0000 == 0 {
            // 16-bit data bus
            let val_hi = ((val & 0xffff_0000) >> 16) as u16;
            let val_lo = (val & 0x0000_ffff) as u16;

            self.update_addr_bus(addr);
            self.update_data_bus(val_hi as u32);

            self.write_mem_short_internal(addr, val_hi);
            self.cycle_write();

            self.update_data_bus(val_lo as u32);

            self.write_mem_short_internal(addr+2, val_lo);
            self.cycle_write();
        } else {
            self.update_data_bus(val);
            self.update_addr_bus(addr);
            self.write_mem_long(addr, val);
            // TODO: do the true 32bit cpus use 1 read/write cycle for longs?
            self.cycle_write();
        }
    }

    /// Write a byte/word/long (based on the `size`) to address `addr`
    /// 
    /// Cycles:
    /// - `.b: 4(0/1)`
    /// - `.w: 4(0/1)`
    /// - `.l: 8(0/2)`
    fn write_mem_size(&mut self, size: DataSize, addr: u32, val: u32) {
        match size {
            // 1 write cycle
            DataSize::Byte => self.write_mem_byte(addr, val as u8),
            // 1 write cycle
            DataSize::Word => self.write_mem_short(addr, val as u16),
            // 2 write cycles
            DataSize::Long => self.write_mem_long(addr, val),
        }
    }

    /// Cycles: 0(0/0)
    pub fn read_mem_byte_internal(&mut self, addr: u32) -> u8 {
        self.mmu.read_byte(addr & self.addr_bus_mask)
    }

    /// Cycles: 0(0/0)
    pub fn read_mem_short_internal(&mut self, addr: u32) -> u16 {
        self.mmu.read_short(addr & self.addr_bus_mask)
    }

    /// Cycles: 8(2/0)
    pub fn read_mem_long_internal(&mut self, addr: u32) -> u32 {
        self.mmu.read_long(addr & self.addr_bus_mask)
    }

    /// Cycles: 4(1/0)
    fn read_mem_byte(&mut self, addr: u32) -> u8 {
        // Bus is 16 bits wide, so 1 read cycle (same as short) is always performed
        self.update_addr_bus(addr);

        let val = self.read_mem_byte_internal(addr & self.addr_bus_mask);

        self.update_data_bus(val as u32);
        self.cycle_read();

        val
    }

    /// Cycles: 4(1/0)
    fn read_mem_short(&mut self, addr: u32) -> u16 {
        self.update_addr_bus(addr);

        let val = self.read_mem_short_internal(addr & self.addr_bus_mask);

        self.update_data_bus(val as u32);
        self.cycle_read();

        val
    }

    /// Cycles: 8(2/0)
    fn read_mem_long(&mut self, addr: u32) -> u32 {
        if self.data_bus_mask & 0x8000_0000 == 0 {
            // 16-bit data bus
            self.update_addr_bus(addr);

            let val_hi = self.read_mem_short_internal(addr & self.addr_bus_mask) as u32;

            self.update_data_bus(val_hi);
            self.cycle_read();

            let val_lo = self.read_mem_short_internal((addr+2) & self.addr_bus_mask) as u32;

            self.update_data_bus(val_lo);
            self.cycle_read();

            (val_hi << 16) | val_lo
        } else {
            self.update_addr_bus(addr);
            let val = self.read_mem_long_internal(addr);
            self.update_data_bus(val);
            // TODO: do the true 32bit cpus use 1 read/write cycle for longs?
            self.cycle_read();

            val
        }
    }

    /// Read a byte/word/long (based on the `size`) at address `addr`
    /// 
    /// Cycles:
    /// - `.b: 4(1/0)`
    /// - `.w: 4(1/0)`
    /// - `.l: 8(2/0)`
    fn read_mem_size(&mut self, size: DataSize, addr: u32) -> u32 {
        match size {
            // 1 read cycle
            DataSize::Byte => self.read_mem_byte(addr) as u32,
            // 1 read cycle
            DataSize::Word => self.read_mem_short(addr) as u32,
            // 2 read cycles
            DataSize::Long => self.read_mem_long(addr),
        }
    }

    /// Read a brief extension word
    ///
    /// Cycles: 4(1/0)
    fn read_extension_brief(&mut self) -> (bool, RegisterNum, DataSize, u8, u8) {
        // TODO: full extension words (used in 68020, 68030, and 68040)
        let w = self.read_pc_inc_short();

        let mode = w & 0x8000 != 0;
        let xn: RegisterNum = (((w & 0x7000) >> 12) as u8).into();
        let size: DataSize = (((w & 0x0800) >> 11) as u8 + 1).into();

        // Used in cpu32, 68020, 68030, 68040
        let scale_factor = ((w & 0x0700) >> 8) as u8;

        let displacement = (w & 0x00ff) as u8;

        (mode, xn, size, scale_factor, displacement)
    }

    fn calculate_am_operand(&mut self, size: DataSize, am: AddressingMode) -> u32 {
        match am {
            // Cycle counts c(r/w)
            // .b, .w | .l
            // 0(0/0) | 0(0/0)
            AddressingMode::DataReg(_) => 0,
            // 0(0/0) | 0(0/0)
            AddressingMode::AddrReg(_) => 0,
            // 4(1/0) | 8(2/0)
            AddressingMode::Addr(an) => self.reg.read_address(an),
            // 4(1/0) | 8(2/0)
            AddressingMode::AddrPostInc(an) => {
                let r = self.reg.read_address(an);
                self.reg.inc_address(an, size);

                r
            },
            // 6(1/0) | 10(2/0)
            AddressingMode::AddrPreDec(an) => {
                self.reg.dec_address(an, size);
                let r = self.reg.read_address(an);

                // ?
                self.cycle(2);

                r
            },
            // 8(2/0) | 12(3/0)
            AddressingMode::AddrDisplace(an) => {
                let r = self.reg.read_address(an);
                let d = sign_extend(DataSize::Long, self.read_pc_inc_short() as u32);
                let addr = r + d;
                addr
            },
            // 10(2/0)| 14(3/0)
            AddressingMode::AddrIdx(an) => {
                // TODO: other indexed addressing modes
                let a_val = self.reg.read_address(an);
                let (x, xn, s, _, d) = self.read_extension_brief();
                let displacement = sign_extend_byte(d);

                let x_reg_val = if x {
                    // address register
                    self.reg.read_address(xn)
                } else {
                    // data register
                    self.reg.read_data(xn)
                };

                let x_val = match s {
                    DataSize::Word => sign_extend(DataSize::Long, x_reg_val),
                    DataSize::Long => x_reg_val,
                    _ => unreachable!()
                };

                // ?
                self.cycle(2);

                let addr = a_val + displacement + x_val;

                addr
            },
            // 8(2/0) | 12(3/0)
            AddressingMode::PCDisplace => {
                let pc = self.reg.pc;
                let d = sign_extend(DataSize::Long, self.read_pc_inc_short() as u32);
                let addr = pc + d;

                addr
            },
            // 10(2/0)| 14(3/0)
            AddressingMode::PCIdx => {
                // TODO: other indexed addressing modes
                let pc_val = self.reg.pc;

                let (x, xn, s, _, d) = self.read_extension_brief();
                let displacement = sign_extend_byte(d);

                let x_reg_val = if x {
                    // address register
                    self.reg.read_address(xn)
                } else {
                    // data register
                    self.reg.read_data(xn)
                };

                let x_val = match s {
                    DataSize::Word => sign_extend(DataSize::Long, x_reg_val),
                    DataSize::Long => x_reg_val,
                    _ => unreachable!()
                };

                // ?
                self.cycle(2);

                let addr = pc_val + displacement + x_val;

                addr
            },
            // 8(2/0) | 12(3/0)
            AddressingMode::AbsShort => {
                // ?
                let addr = sign_extend(DataSize::Long, self.read_pc_inc_short() as u32);

                addr
            },
            // 12(3/0)| 16(4/0)
            AddressingMode::AbsLong => {
                let addr = self.read_pc_inc_long();

                addr
            },
            // 4(1/0) | 8(2/0)
            AddressingMode::Imm => {
                0
            },
            AddressingMode::Invalid => 0,
        }
    }

    /// Perform a read using the specified addressing mode and with the specified size.
    /// 
    /// Cycles:
    /// 
    /// |        |.b, .w | .l     |
    /// |--------|-------|--------|
    /// |Dn      |0(0/0) | 0(0/0) |
    /// |An      |0(0/0) | 0(0/0) |
    /// |(An)    |4(1/0) | 8(2/0) |
    /// |(An)+   |4(1/0) | 8(2/0) |
    /// |-(An)   |6(1/0) | 10(2/0)|
    /// |d(An)   |8(2/0) | 12(3/0)|
    /// |d(An,ix)|10(2/0)| 14(3/0)|
    /// |xxx.w   |8(2/0) | 12(3/0)|
    /// |xxx.l   |10(2/0)| 14(3/0)|
    /// |d(PC)   |8(2/0) | 12(3/0)|
    /// |d(PC,ix)|12(3/0)| 16(4/0)|
    /// |#xxx    |4(1/0) | 8(2/0) |
    pub fn read_am(&mut self, size: DataSize, am: AddressingMode) -> u32 {
        let addr = self.calculate_am_operand(size, am);

        match am {
            // Cycle counts c(r/w)
            // .b, .w | .l
            // 0(0/0) | 0(0/0)
            AddressingMode::DataReg(dn) => self.reg.read_data(dn),
            // 0(0/0) | 0(0/0)
            AddressingMode::AddrReg(an) => self.reg.read_address(an),
            // 4(1/0) | 8(2/0)
            AddressingMode::Imm => {
                match size {
                    DataSize::Byte => self.read_pc_inc_short() as u32 & 0xff,
                    DataSize::Word => self.read_pc_inc_short() as u32,
                    DataSize::Long => self.read_pc_inc_long(),
                }
            },
            AddressingMode::Invalid => panic!("Cannot read with invalid addressing mode"),
            _ => self.read_mem_size(size, addr)
        }
    }

    /// Perform a write using the specified addressing mode and with the specified size.
    /// 
    /// Cycles:
    /// 
    /// |        |.b, .w | .l     |
    /// |--------|-------|--------|
    /// |Dn      |0(0/0) | 0(0/0) |
    /// |An      |0(0/0) | 0(0/0) |
    /// |(An)    |4(1/0) | 8(2/0) |
    /// |(An)+   |4(1/0) | 8(2/0) |
    /// |-(An)   |6(1/0) | 10(2/0)|
    /// |d(An)   |8(2/0) | 12(3/0)|
    /// |d(An,ix)|10(2/0)| 14(3/0)|
    /// |xxx.w   |8(2/0) | 12(3/0)|
    /// |xxx.l   |10(2/0)| 14(3/0)|
    /// |d(PC)   |8(2/0) | 12(3/0)|
    /// |d(PC,ix)|12(3/0)| 16(4/0)|
    /// |#xxx    |4(1/0) | 8(2/0) |
    pub fn write_am(&mut self, size: DataSize, am: AddressingMode, val: u32) {
        let addr = self.calculate_am_operand(size, am);

        match am {
            // 0(0/0)
            AddressingMode::DataReg(dn) => self.reg.write_data(dn, size, val),
            // 0(0/0)
            AddressingMode::AddrReg(an) => self.reg.write_address(an, size, val),
            // x(n/a)
            AddressingMode::Imm => panic!("Cannot write to immediate"),
            AddressingMode::Invalid => panic!("Cannot write with invalid addressing mode"),
            // 4(0/1) | 8(0/2)
            _ => self.write_mem_size(size, addr, val)
        }
    }


    /// Push an element on top of the stack.
    /// Decreases the user stack pointer (a7) by 2.
    /// 
    /// ```p
    /// SP - 2 -> SP
    /// val -> (SP);
    /// ```
    /// 
    /// Cycles: 4(0/1)
    pub fn push_sp_short(&mut self, val: u16) {
        self.reg.dec_address(RegisterNum::R7, DataSize::Word);
        self.write_mem_short(self.reg.read_address(RegisterNum::R7), val)
    }

    /// Push an element on top of the stack.
    /// Decreases the user stack pointer (a7) by 4.
    /// 
    /// ```p
    /// SP - 4 -> SP
    /// val -> (SP);
    /// ```
    /// 
    /// Cycles: 8(0/2)
    pub fn push_sp_long(&mut self, val: u32) {
        self.reg.dec_address(RegisterNum::R7, DataSize::Long);
        self.write_mem_long(self.reg.read_address(RegisterNum::R7), val)
    }

    /// Pop an element from the top of the stack.
    /// Increases the user stack pointer (a7) by 4.
    /// 
    /// ```p
    /// (SP) -> val;
    /// SP + 2 -> SP
    /// ```
    /// 
    /// Cycles: 4(2/0)
    pub fn pop_sp_short(&mut self) -> u16 {
        let res = self.read_mem_short(self.reg.read_address(RegisterNum::R7));
        self.reg.inc_address(RegisterNum::R7, DataSize::Word);
        res
    }

    /// Pop an element from the top of the stack.
    /// Increases the user stack pointer (a7 ) by 4.
    /// 
    /// ```p
    /// (SP) -> val;
    /// SP + 4 -> SP
    /// ```
    /// 
    /// Cycles: 8(2/0)
    pub fn pop_sp_long(&mut self) -> u32 {
        let res = self.read_mem_long(self.reg.read_address(RegisterNum::R7));
        self.reg.inc_address(RegisterNum::R7, DataSize::Long);
        res
    }


    /// Cycles: 4(1/0)
    pub fn read_pc_short(&mut self) -> u16 {
        let val = self.read_mem_short(self.reg.pc);
        val
    }

    /// Cycles: 4(1/0)
    pub fn read_pc_inc_short(&mut self) -> u16 {
        let val = self.read_mem_short(self.reg.pc);
        self.reg.pc += 2;
        val
    }

    /// Cycles: 8(2/0)
    pub fn read_pc_long(&mut self) -> u32 {
        self.read_mem_long(self.reg.pc)
    }

    /// Cycles: 8(2/0)
    pub fn read_pc_inc_long(&mut self) -> u32 {
        if self.data_bus_mask & 0x8000_0000 == 0 {
            let val_hi = self.read_pc_inc_short() as u32;
            let val_lo = self.read_pc_inc_short() as u32;
            (val_hi << 16) | val_lo
        } else {
            let val = self.read_pc_long();
            self.reg.pc += 2;
            val
        }
    }

    fn fetch_and_run(&mut self) {
        let op = self.read_pc_inc_short();
        let inst = inst::decode_opcode(op);
        println!("  Running instruction: {inst} ({op:016b})");
        inst.run(self);
    }

    pub fn run(&mut self) -> bool {
        if self.stopped {
            println!("Stopping execution.");
            return false;
        }

        println!();
        let pre_cycles = self.cycles;
        let pre_cycles_read = self.read_cycles;
        let pre_cycles_write= self.write_cycles;
        self.fetch_and_run();
        let post_cycles = self.cycles;
        let post_cycles_read = self.read_cycles;
        let post_cycles_write= self.write_cycles;
        self.print_state();
        println!(
            "TOOK CYCLES: {}({}/{})",
            post_cycles - pre_cycles,
            post_cycles_read - pre_cycles_read,
            post_cycles_write - pre_cycles_write
        );

        true
    }

    fn cycle(&mut self, amount: usize) {
        self.cycles += amount;
    }

    fn cycle_read(&mut self) {
        self.cycle(4);
        self.read_cycles += 1;
    }

    fn cycle_write(&mut self) {
        self.cycle(4);
        self.write_cycles += 1;
    }

    /// Cycle only when size is .l
    fn cycle_when_long(&mut self, size: DataSize, amount: usize) {
        match size {
            DataSize::Long => self.cycle(amount),
            _ => {}
        }
    }
}
