use crate::{bits::{DataSize, SignExtend, is_negative, is_zero, sign_extend_size}, cpu::{CPU, CPU_VECTOR_SIZE, CPURevision::{self, MC68000, MC68010}, CPUVector, RegisterNum, opcode::*}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Condition {
    T,
    F,
    HI,
    LS,
    CC,
    CS,
    NE,
    EQ,
    VC,
    VS,
    PL,
    MI,
    GE,
    LT,
    GT,
    LE,
}

impl Condition {
    fn is_true(&self, cpu: &mut CPU) -> bool {
        match self {
            Condition::T  => true,
            Condition::F  => false,
            Condition::HI => !cpu.reg.sr.c() && !cpu.reg.sr.z(),
            Condition::LS => cpu.reg.sr.c() || cpu.reg.sr.z(),
            Condition::CC => !cpu.reg.sr.c(),
            Condition::CS => cpu.reg.sr.c(),
            Condition::NE => !cpu.reg.sr.z(),
            Condition::EQ => cpu.reg.sr.z(),
            Condition::VC => !cpu.reg.sr.v(),
            Condition::VS => cpu.reg.sr.v(),
            Condition::PL => !cpu.reg.sr.n(),
            Condition::MI => cpu.reg.sr.n(),
            Condition::GE => cpu.reg.sr.n() && cpu.reg.sr.v() || !cpu.reg.sr.n() && !cpu.reg.sr.v(),
            Condition::LT => cpu.reg.sr.n() && !cpu.reg.sr.v() || !cpu.reg.sr.n() && cpu.reg.sr.v(),
            Condition::GT => {
                cpu.reg.sr.n() && cpu.reg.sr.v() && !cpu.reg.sr.z()
                || !cpu.reg.sr.n() && !cpu.reg.sr.v() && !cpu.reg.sr.z()
            },
            Condition::LE => {
                cpu.reg.sr.z()
                || cpu.reg.sr.n() && !cpu.reg.sr.v()
                || !cpu.reg.sr.n() && cpu.reg.sr.v()
            },
        }
    }
}

impl From<u16> for Condition {
    fn from(value: u16) -> Self {
        use Condition::*;

        assert!(value <= 0b1111, "Condition must be <= 0b1111.");

        match value {
            0 => T,
            1 => F,
            2 => HI,
            3 => LS,
            4 => CC,
            5 => CS,
            6 => NE,
            7 => EQ,
            8 => VC,
            9 => VS,
            10 => PL,
            11 => MI,
            12 => GE,
            13 => LT,
            14 => GT,
            15 => LE,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::T  => write!(f, "t"),
            Condition::F  => write!(f, "f"),
            Condition::HI => write!(f, "hi"),
            Condition::LS => write!(f, "ls"),
            Condition::CC => write!(f, "cc"),
            Condition::CS => write!(f, "cs"),
            Condition::NE => write!(f, "ne"),
            Condition::EQ => write!(f, "eq"),
            Condition::VC => write!(f, "vc"),
            Condition::VS => write!(f, "vs"),
            Condition::PL => write!(f, "pl"),
            Condition::MI => write!(f, "mi"),
            Condition::GE => write!(f, "ge"),
            Condition::LT => write!(f, "lt"),
            Condition::GT => write!(f, "gt"),
            Condition::LE => write!(f, "le"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    /// Data register `Dn`
    DataReg(RegisterNum),
    /// Address register `An`
    AddrReg(RegisterNum),
    /// Indirect address `(An)`
    Addr(RegisterNum),
    /// Address with postincrement `(An)+`
    AddrPostInc(RegisterNum),
    /// Address with predecrement `-(An)`
    AddrPreDec(RegisterNum),
    /// Address with displacement `(d16, An)`
    AddrDisplace(RegisterNum),
    /// Address with index `(d8, An, Xn)`
    /// - AddrRegIndIdx8 requires 1 extension word
    /// - AddrRegIndIdxBase requires 1,2, or 3 extension word(s)
    /// - MemIndPostIdx requires 1,2,3,4, or 5 extension word(s)
    /// - MemIndPreIdx requires 1,2,3,4, or 5 extension word(s)
    AddrIdx(RegisterNum),
    /// Program Counter with displacement `(d16, PC)`
    PCDisplace,
    /// Program Counter with index `(d8, PC, Xn)`
    /// 
    /// Either of 8bit displacement or base displacement
    PCIdx,
    /// Absolute word `(xxx).W`
    AbsWord,
    /// Absolute long `(xxx).L`
    AbsLong,
    /// Immediate `#imm`
    Imm,
    /// A mode that does not exist.
    Invalid,
}

#[cfg(feature = "std")]
impl std::fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode::DataReg(r)      => write!(f, "d{}", Into::<u8>::into(*r)),
            AddressingMode::AddrReg(r)      => write!(f, "a{}", Into::<u8>::into(*r)),
            AddressingMode::Addr(r)         => write!(f, "(a{})", Into::<u8>::into(*r)),
            AddressingMode::AddrPostInc(r)  => write!(f, "(a{})+", Into::<u8>::into(*r)),
            AddressingMode::AddrPreDec(r)   => write!(f, "-(a{})", Into::<u8>::into(*r)),
            AddressingMode::AddrDisplace(r) => write!(f, "d(a{})", Into::<u8>::into(*r)),
            AddressingMode::AddrIdx(r)      => write!(f, "d(a{},Xn)", Into::<u8>::into(*r)),
            AddressingMode::PCDisplace                    => write!(f, "d(pc)"),
            AddressingMode::PCIdx                         => write!(f, "d(pc,Xn)"),
            AddressingMode::AbsWord                      => write!(f, "(xxx).w"),
            AddressingMode::AbsLong                       => write!(f, "(xxx).l"),
            AddressingMode::Imm                           => write!(f, "#imm"),
            AddressingMode::Invalid                       => write!(f, "(invalid addressing mode)"),
        }
    }
}

impl AddressingMode {
    pub(crate) fn decode(mode: u8, reg: u8) -> AddressingMode {
        use AddressingMode::*;

        assert_eq!(mode & 0b111, mode);
        assert_eq!(reg  & 0b111, reg);

        let reg_n: RegisterNum = reg.into();

        match mode {
            0b000 => DataReg(reg_n),
            0b001 => AddrReg(reg_n),
            0b010 => Addr(reg_n),
            0b011 => AddrPostInc(reg_n),
            0b100 => AddrPreDec(reg_n),
            0b101 => AddrDisplace(reg_n),
            0b110 => AddrIdx(reg_n),
            0b111 => {
                match reg {
                    0b000 => AbsWord,
                    0b001 => AbsLong,
                    0b010 => PCDisplace,
                    0b011 => PCIdx,
                    0b100 => Imm,
                    _ => Invalid,
                }
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn decode_op(op: u16) -> AddressingMode {
        let mode = get_op_m_lo(op);
        let reg  = get_op_xn_lo(op);
        AddressingMode::decode(mode, reg)
    }
}

#[derive(Debug)]
pub enum Instruction {
    // 0000
    ORIToCCR,
    ORIToSR,
    ORI   { op: u16 },
    ANDIToCCR,
    ANDIToSR,
    ANDI  { op: u16 },
    SUBI  { op: u16 },
    ADDI  { op: u16 },
    EORIToCCR,
    EORIToSR,
    EORI  { op: u16 },
    CMPI  { op: u16 },
    BTST  { op: u16 },
    BCHG  { op: u16 },
    BCLR  { op: u16 },
    BSET  { op: u16 },
    MOVEP { op: u16 },
    // 00ss
    MOVEA { op: u16 },
    MOVE  { op: u16 },
    // 0100
    MOVEFromSR { op: u16 },
    MOVEToCCR  { op: u16 },
    MOVEToSR   { op: u16 },
    NEGX { op: u16 },
    CLR  { op: u16 },
    NEG  { op: u16 },
    NOT  { op: u16 },
    EXT  { op: u16 },
    NBCD { op: u16 },
    SWAP { op: u16 },
    PEA  { op: u16 },
    ILLEGAL,
    TAS  { op: u16 },
    TST  { op: u16 },
    TRAP { op: u16 },
    LINK { op: u16 },
    UNLK { op: u16 },
    MOVEUSP { op: u16 },
    RESET, NOP, STOP,
    RTE, RTS, TRAPV, RTR,
    JSR   { op: u16 },
    JMP   { op: u16 },
    MOVEM { op: u16 },
    LEA   { op: u16 },
    CHK   { op: u16 },
    // 0101
    ADDQ { op: u16 },
    SUBQ { op: u16 },
    Scc  { op: u16 },
    DBcc { op: u16 },
    // 0110
    BRA { op: u16 },
    BSR { op: u16 },
    Bcc { op: u16 },
    // 0111
    MOVEQ { op: u16 },
    // 1000
    DIVU { op: u16 },
    DIVS { op: u16 },
    SBCD { op: u16 },
    OR   { op: u16 },
    // 1001
    SUB  { op: u16 },
    SUBX { op: u16 },
    SUBA { op: u16 },
    // 1010
    // ...
    // 1011
    EOR  { op: u16 },
    CMPM { op: u16 },
    CMP  { op: u16 },
    CMPA { op: u16 },
    // 1100
    MULU { op: u16 },
    MULS { op: u16 },
    ABCD { op: u16 },
    EXG  { op: u16 },
    AND  { op: u16 },
    // 1101
    ADD  { op: u16 },
    ADDX { op: u16 },
    ADDA { op: u16 },
    // 1110
    // (shift/rotate memory)
    ASdM { op: u16 },
    LSdM { op: u16 },
    ROXdM{ op: u16 },
    ROdM { op: u16 },
    // (shift/rotate data reg/immediate)
    ASdD { op: u16 },
    LSdD { op: u16 },
    ROXdD{ op: u16 },
    ROdD { op: u16 },
    // 1111
    // Floating point/supervisor instructions?

    // Extra
    Undefined { op: u16 },
}

#[cfg(feature = "std")]
macro_rules! display_a_op_b {
    ($f:expr, $inst:expr, $op:expr) => {
        {
            let am = AddressingMode::decode_op(*$op);
            let m = get_op_m_hi(*$op);
            let s: DataSize = (m & 0b11).into();
            let dn = get_op_xn_hi(*$op);
            if m & 0b100 == 0 {
                // <ea> + Dn -> Dn
                write!($f, "{}{} {},d{}", $inst, s, am, dn)
            } else {
                // Dn + <ea> -> <ea>
                write!($f, "{}{} d{},{}", $inst, s, dn, am)
            }
        }
    };
}

#[cfg(feature = "std")]
macro_rules! display_bop {
    ($f:expr, $inst:expr, $op:expr) => {
        {
            let am = AddressingMode::decode_op(*$op);
            let xn = get_op_xn_hi(*$op);

            let s = match am {
                AddressingMode::DataReg(_) => DataSize::Long,
                _ => DataSize::Byte,
            };

            if xn != 0b100 {
                // dn,<ea>
                write!($f, "{}{} d{},{}", $inst, s, xn, am)
            } else {
                // #<data>,<ea>
                write!($f, "{}{} #imm,{}", $inst, s, am)
            }
        } 
    };
}

#[cfg(feature = "std")]
macro_rules! display_shift_m {
    ($f:expr, $inst:expr, $op:expr) => {
        {
            let am = AddressingMode::decode_op(*$op);
            let d = if *$op & 0b100_000_000 != 0 { "l" } else { "r" };

            write!($f, "{}{}.w {}", $inst, d, am)
        }
    };
}

#[cfg(feature = "std")]
macro_rules! display_shift_d {
    ($f:expr, $inst:expr, $op:expr) => {
        {
            let m = (*$op & 0b100_000) != 0;
            let s = get_op_size(*$op);
            let d = if (*$op & 0b100_000_000) != 0 { "l" } else { "r" };
            let dn_to_shift = get_op_xn_lo(*$op);
            let mut count = get_op_xn_hi(*$op);

            if m {
                // Dx,Dy   ; Dy << Dx
                write!($f, "{}{}{} d{},d{}", $inst, d, s, count, dn_to_shift)
            } else {
                // #imm,Dy ; Dy << #imm
                if count == 0 {
                    count = 8;
                }

                write!($f, "{}{}{} #{},d{}", $inst, d, s, count, dn_to_shift)
            }
        }
    };
}

#[cfg(feature = "std")]
impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;
        match self {
            // moves
            MOVEP { op } => {
                let s = get_op_size_bit6(*op);
                let an = get_op_xn_lo(*op);
                let dn = get_op_xn_hi(*op);
                if get_op_m_hi(*op) & 0b010 != 0 {
                    // reg -> mem
                    write!(f, "movep{} d{},(d,a{})", s, dn, an)
                } else {
                    // mem -> reg
                    write!(f, "movep{} (d,a{}),d{}", s, an, dn)
                }
            },
            MOVEA { op } => {
                let s = get_op_size_alt(*op, SIZE_SHIFT_MOVE);

                let am_src = AddressingMode::decode_op(*op);

                let xn = get_op_xn_hi(*op);

                write!(f, "movea{} {},a{}", s, am_src, xn)
            },
            MOVE  { op } => {
                let s = get_op_size_alt(*op, SIZE_SHIFT_MOVE);

                let am_src = AddressingMode::decode_op(*op);

                let m = get_op_m_hi(*op);
                let xn = get_op_xn_hi(*op);
                let am_dst = AddressingMode::decode(m, xn);

                write!(f, "move{} {},{}", s, am_src, am_dst)
            },
            MOVEM { op } => {
                let s = get_op_size_bit6(*op);
                let am = AddressingMode::decode_op(*op);
                if op & 0x400 != 0 {
                    // mem -> reg
                    write!(f, "movem{} {},list", s, am)
                } else {
                    // reg -> mem
                    write!(f, "movem{} list,{}", s, am)
                }
            },
            MOVEQ { op } => write!(f, "moveq #0x{:02x},d{}", op & 0xff, get_op_xn_hi(*op)),
            MOVEFromSR{ op } => write!(f, "move.w sr,{}", AddressingMode::decode_op(*op)),
            MOVEToCCR { op } => write!(f, "move.w {},ccr", AddressingMode::decode_op(*op)),
            MOVEToSR  { op } => write!(f, "move.w {},sr", AddressingMode::decode_op(*op)),
            MOVEUSP {op} => {
                let an = get_op_xn_lo(*op);
                if op & 0b1000 == 0 {
                    // 0: an -> usp
                    write!(f, "move.l a{},usp", an)
                } else {
                    // 1: usp -> an
                    write!(f, "move.l usp,a{}", an)
                }
            },
            // arithmetic
            ADDI  { op } => {
                let am = AddressingMode::decode_op(*op);
                let s = get_op_size(*op);

                write!(f, "addi{} #imm,{}", s, am)
            },
            ADDQ  { op } => {
                let s = get_op_size(*op);
                let am = AddressingMode::decode_op(*op);
                let val = ((*op) & 0xe00) >> 9;
                write!(f, "addq{} #{:01x},{}", s, val, am)
            },
            ADD   { op } => display_a_op_b!(f, "add", op),
            ADDX  { op } => {
                let xn_src = get_op_xn_lo(*op);
                let xn_dst = get_op_xn_hi(*op);
                let s = get_op_size(*op);
                let m = op & 0b1000 == 0;

                if m {
                    // ADDX Dy,Dx
                    write!(f, "addx{} d{},d{}", s, xn_src, xn_dst)
                } else {
                    // ADDX -(Ay),-(Ax)
                    write!(f, "addx{} -(a{}),-(a{})", s, xn_src, xn_dst)
                }
            },
            ADDA  { op } => {
                let am = AddressingMode::decode_op(*op);
                let s = get_op_size_bit8(*op);
                let an = get_op_xn_hi(*op);
                write!(f, "adda{} {},a{}", s, am, an)
            },
            SUBI  { op } => {
                let am = AddressingMode::decode_op(*op);
                let s = get_op_size(*op);

                write!(f, "subi{} #imm,{}", s, am)
            },
            SUBQ  { op } => {
                let s = get_op_size(*op);
                let am = AddressingMode::decode_op(*op);
                let val = ((*op) & 0xe00) >> 9;
                write!(f, "subq{} #{:01x},{}", s, val, am)
            },
            SUB   { op } => display_a_op_b!(f, "sub", op),
            SUBX  { op } => {
                let xn_src = get_op_xn_lo(*op);
                let xn_dst = get_op_xn_hi(*op);
                let s = get_op_size(*op);
                let m = op & 0b1000 == 0;

                if m {
                    // ADDX Dy,Dx
                    write!(f, "subx{} d{},d{}", s, xn_src, xn_dst)
                } else {
                    // ADDX -(Ay),-(Ax)
                    write!(f, "subx{} -(a{}),-(a{})", s, xn_src, xn_dst)
                }
            },
            SUBA  { op } => {
                let am = AddressingMode::decode_op(*op);
                let s = get_op_size_bit8(*op);
                let an = get_op_xn_hi(*op);
                write!(f, "suba{} {},a{}", s, am, an)
            },
            DIVU  { op } => write!(f, "divu.w {},d{}", AddressingMode::decode_op(*op), get_op_xn_hi(*op)),
            DIVS  { op } => write!(f, "divs.w {},d{}", AddressingMode::decode_op(*op), get_op_xn_hi(*op)),
            MULU  { op } => write!(f, "mulu.w {},d{}", AddressingMode::decode_op(*op), get_op_xn_hi(*op)),
            MULS  { op } => write!(f, "muls.w {},d{}", AddressingMode::decode_op(*op), get_op_xn_hi(*op)),
            // exceptions, etc.
            TRAP  { op } => write!(f, "trap #{}", op & 0xf),
            TRAPV              => write!(f, "trapv"),
            ILLEGAL            => write!(f, "illegal"),
            CHK   { op } => {
                let s = get_op_size_alt(*op, SIZE_SHIFT_CHK);
                let am = AddressingMode::decode_op(*op);
                let dn = get_op_xn_hi(*op);
                write!(f, "chk{} {},d{}", s, am, dn)
            },
            RESET              => write!(f, "reset"),
            STOP               => write!(f, "stop #imm"),
            // jumps
            JSR   { op } => write!(f, "jsr {}", AddressingMode::decode_op(*op)),
            JMP   { op } => write!(f, "jmp {}", AddressingMode::decode_op(*op)),
            // branch
            DBcc  { op } => {
                let cc: Condition = ((op & 0x0f00) >> 8).into();
                let dn = get_op_xn_lo(*op);

                write!(f, "db{} d{},displace", cc, dn)
            },
            BRA   { op } => {
                let displace8bit = op & 0xff;
                let s = match displace8bit {
                    0x00 => DataSize::Word,
                    0xff => DataSize::Long,
                    _ => DataSize::Byte
                };
                write!(f, "bra{} displace", s)
            },
            BSR   { op } => {
                let displace8bit = op & 0xff;
                let s = match displace8bit {
                    0x00 => DataSize::Word,
                    0xff => DataSize::Long,
                    _ => DataSize::Byte
                };
                write!(f, "bsr{} displace", s)
            },
            Bcc   { op } => {
                let displace8bit = op & 0xff;
                let s = match displace8bit {
                    0x00 => DataSize::Word,
                    0xff => DataSize::Long,
                    _ => DataSize::Byte
                };
                let cc: Condition = ((op & 0x0f00) >> 8).into();
                write!(f, "b{}{} displace", cc, s)
            },
            // returns
            RTE                => write!(f, "rte"),
            RTS                => write!(f, "rts"),
            RTR                => write!(f, "rtr"),
            // link
            LINK  { op } => {
                let an = get_op_xn_lo(*op);
                let size = if op & 0b0000011001010000 != 0 {
                    DataSize::Word
                } else {
                    DataSize::Long
                };
                write!(f, "link{} a{},#imm", size, an)
            },
            UNLK  { op } => write!(f, "unlk a{}", get_op_xn_lo(*op)),
            // <ea>
            LEA   { op } => write!(f, "lea {}", AddressingMode::decode_op(*op)),
            PEA   { op } => write!(f, "pea {}", AddressingMode::decode_op(*op)),
            // logical
            ORI   { op } => write!(f, "ori{} #imm,{}", get_op_size(*op), AddressingMode::decode_op(*op)),
            ANDI  { op } => write!(f, "andi{} #imm,{}", get_op_size(*op), AddressingMode::decode_op(*op)),
            EORI  { op } => write!(f, "eori{} #imm,{}", get_op_size(*op), AddressingMode::decode_op(*op)),
            OR    { op } => display_a_op_b!(f, "or", op),
            AND   { op } => display_a_op_b!(f, "and", op),
            EOR   { op } => display_a_op_b!(f, "eor", op),
            ORIToCCR  => write!(f, "ori.b #imm,ccr"),
            ORIToSR   => write!(f, "ori.w #imm,sr"),
            ANDIToCCR => write!(f, "andi.b #imm,ccr"),
            ANDIToSR  => write!(f, "andi.w #imm,sr"),
            EORIToCCR => write!(f, "eori.b #imm,ccr"),
            EORIToSR  => write!(f, "eori.w #imm,sr"),
            CMP   { op } => {
                let s = get_op_size(*op);
                let am = AddressingMode::decode_op(*op);
                let dn = get_op_xn_hi(*op);
                write!(f, "cmp{} {},d{}", s, am, dn)
            },
            CMPI  { op } => {
                let s = get_op_size(*op);
                let am = AddressingMode::decode_op(*op);
                write!(f, "cmpi{} #imm,{}", s, am)
            },
            CMPA  { op } => {
                let s = get_op_size_bit8(*op);
                let am = AddressingMode::decode_op(*op);
                let an = get_op_xn_hi(*op);
                write!(f, "cmpa{} {},a{}", s, am, an)
            },
            CMPM  { op } => {
                let s = get_op_size(*op);
                let ay = get_op_xn_lo(*op);
                let ax = get_op_xn_hi(*op);
                write!(f, "cmpm{} (a{})+,(a{})+", s, ay, ax)
            },
            // logical (single)
            NEGX  { op } => write!(f, "negx{} {}", get_op_size(*op), AddressingMode::decode_op(*op)),
            NEG   { op } => write!(f, "neg{} {}", get_op_size(*op), AddressingMode::decode_op(*op)),
            NOT   { op } => write!(f, "not{} {}", get_op_size(*op), AddressingMode::decode_op(*op)),
            // bit
            BTST  { op } => display_bop!(f, "btst", op),
            BCHG  { op } => display_bop!(f, "bchg", op),
            BCLR  { op } => display_bop!(f, "bclr", op),
            BSET  { op } => display_bop!(f, "bset", op),
            TAS   { op } => write!(f, "tas {}", AddressingMode::decode_op(*op)),
            TST   { op } => write!(f, "tst{} {}", get_op_size(*op), AddressingMode::decode_op(*op)),
            // shift/rotate
            ASdM   { op } => display_shift_m!(f, "as", op),
            LSdM   { op } => display_shift_m!(f, "ls", op),
            ROXdM  { op } => display_shift_m!(f, "rox", op),
            ROdM   { op } => display_shift_m!(f, "ro", op),
            ASdD   { op } => display_shift_d!(f, "as", op),
            LSdD   { op } => display_shift_d!(f, "ls", op),
            ROXdD  { op } => display_shift_d!(f, "rox", op),
            ROdD   { op } => display_shift_d!(f, "ro", op),
            // bcd
            ABCD  { op } => {
                let xx = get_op_xn_hi(*op);
                let xy = get_op_xn_lo(*op);
                let m = op & 0b1000 == 0;

                if m {
                    write!(f, "abcd d{},d{}", xy, xx)
                } else {
                    write!(f, "abcd -(a{}),-(a{})", xy, xx)
                }
            },
            SBCD  { op } => {
                let xx = get_op_xn_hi(*op);
                let xy = get_op_xn_lo(*op);
                let m = op & 0b1000 == 0;

                if m {
                    write!(f, "sbcd d{},d{}", xy, xx)
                } else {
                    write!(f, "sbcd -(a{}),-(a{})", xy, xx)
                }
            },
            NBCD  { op } => write!(f, "nbcd {}", AddressingMode::decode_op(*op)),
            // register modification
            EXT   { op } => write!(f, "ext{} d{}", get_op_size_bit6(*op), get_op_xn_lo(*op)),
            SWAP  { op } => write!(f, "swap.w d{}", get_op_xn_lo(*op)),
            EXG   { op } => {
                let xy = get_op_xn_lo(*op);
                let xx = get_op_xn_hi(*op);
                let m_hi = (op & 0b11_000_000) >> 6;
                let m_lo = (op & 0b1_000) != 0;

                if m_hi == 0b01 && !m_lo {
                    // Dx <-> Dy
                    write!(f, "exg d{},d{}", xx, xy)
                } else if m_hi == 0b01 {
                    // Ax <-> Ay
                    write!(f, "exg a{},a{}", xx, xy)
                } else {
                    // Dx <-> Ay
                    write!(f, "exg d{},a{}", xx, xy)
                }
            },
            // clear/set
            CLR   { op } => write!(f, "clr{} {}", get_op_size(*op), AddressingMode::decode_op(*op)),
            Scc   { op } => {
                let cc: Condition = ((op & 0x0f00) >> 8).into();
                let am = AddressingMode::decode_op(*op);

                write!(f, "s{} {}", cc, am)
            },
            // other
            NOP                => write!(f, "nop"),
            Undefined{op}=> write!(f, "undefined 0x{op:04x}"),
        }
    }
}

impl Instruction {
    pub(crate) fn run(self, cpu: &mut CPU) {
        match self {
            // move
            Instruction::MOVEP { op } => movep(op, cpu),
            Instruction::MOVEA { op } => movea(op, cpu),
            Instruction::MOVE  { op } => move_(op, cpu),
            Instruction::MOVEQ { op } => moveq(op, cpu),
            Instruction::MOVEM { op } => movem(op, cpu),
            Instruction::MOVEUSP {op} => move_usp(op, cpu),
            Instruction::MOVEFromSR {op} => move_from_sr(op, cpu),
            Instruction::MOVEToCCR  {op} => move_to_ccr(op, cpu),
            Instruction::MOVEToSR   {op} => move_to_sr(op, cpu),
            // arithmetic
            Instruction::ADDI  { op } => addi(op, cpu),
            Instruction::ADD   { op } => add(op, cpu),
            Instruction::ADDX  { op } => addx(op, cpu),
            Instruction::ADDA  { op } => adda(op, cpu),
            Instruction::ADDQ  { op } => addq(op, cpu),
            Instruction::SUBI  { op } => subi(op, cpu),
            Instruction::SUB   { op } => sub(op, cpu),
            Instruction::SUBX  { op } => subx(op, cpu),
            Instruction::SUBA  { op } => suba(op, cpu),
            Instruction::SUBQ  { op } => subq(op, cpu),
            Instruction::DIVU  { op } => div(op, cpu),
            Instruction::DIVS  { op } => div(op, cpu),
            Instruction::MULU  { op } => mul(op, cpu),
            Instruction::MULS  { op } => mul(op, cpu),
            // exceptions, etc.
            Instruction::TRAP  { op } => trap(op, cpu),
            Instruction::TRAPV             => trapv(cpu),
            Instruction::ILLEGAL           => trap_illegal(0x4afc, cpu),
            Instruction::CHK   { op } => chk(op, cpu),
            Instruction::RESET             => reset(cpu),
            Instruction::STOP              => stop(cpu),
            // jumps
            Instruction::JSR   { op } => jmp(op, cpu),
            Instruction::JMP   { op } => jmp(op, cpu),
            // branch
            Instruction::DBcc  { op } => dbcc(op, cpu),
            Instruction::BRA   { op } => bra(op, cpu),
            Instruction::BSR   { op } => bsr(op, cpu),
            Instruction::Bcc   { op } => bcc(op, cpu),
            // returns
            Instruction::RTE               => rte(cpu),
            Instruction::RTS               => rts(cpu),
            Instruction::RTR               => rtr(cpu),
            // link
            Instruction::LINK  { op } => link(op, cpu),
            Instruction::UNLK  { op } => unlk(op, cpu),
            // <ea>
            Instruction::LEA   { op } => lea(op, cpu),
            Instruction::PEA   { op } => pea(op, cpu),
            // logical
            Instruction::ORI   { op } => ori(op, cpu),
            Instruction::ANDI  { op } => andi(op, cpu),
            Instruction::EORI  { op } => eori(op, cpu),
            Instruction::OR    { op } => or(op, cpu),
            Instruction::AND   { op } => and(op, cpu),
            Instruction::EOR   { op } => eor(op, cpu),
            Instruction::ORIToSR           => ori_to_sr(cpu),
            Instruction::ORIToCCR          => ori_to_ccr(cpu),
            Instruction::ANDIToSR          => andi_to_sr(cpu),
            Instruction::ANDIToCCR         => andi_to_ccr(cpu),
            Instruction::EORIToSR          => eori_to_sr(cpu),
            Instruction::EORIToCCR         => eori_to_ccr(cpu),
            Instruction::CMP   { op } => cmp(op, cpu),
            Instruction::CMPI  { op } => cmpi(op, cpu),
            Instruction::CMPA  { op } => cmpa(op, cpu),
            Instruction::CMPM  { op } => cmpm(op, cpu),
            // logical (single)
            Instruction::NEGX  { op } => neg(op, cpu),
            Instruction::NEG   { op } => neg(op, cpu),
            Instruction::NOT   { op } => not(op, cpu),
            // bit
            Instruction::BTST  { op } => btst(op, cpu),
            Instruction::BCHG  { op } => bchg(op, cpu),
            Instruction::BCLR  { op } => bclr(op, cpu),
            Instruction::BSET  { op } => bset(op, cpu),
            Instruction::TAS   { op } => tas(op, cpu),
            Instruction::TST   { op } => tst(op, cpu),
            // shift/rotate
            Instruction::ASdM   { op } => asd_m(op, cpu),
            Instruction::LSdM   { op } => lsd_m(op, cpu),
            Instruction::ROXdM  { op } => roxd_m(op, cpu),
            Instruction::ROdM   { op } => rod_m(op, cpu),
            Instruction::ASdD   { op } => asd_d(op, cpu),
            Instruction::LSdD   { op } => lsd_d(op, cpu),
            Instruction::ROXdD  { op } => roxd_d(op, cpu),
            Instruction::ROdD   { op } => rod_d(op, cpu),
            // bcd
            Instruction::ABCD  { op } => abcd(op, cpu),
            Instruction::SBCD  { op } => sbcd(op, cpu),
            Instruction::NBCD  { op } => nbcd(op, cpu),
            // register modification
            Instruction::EXT   { op } => ext(op, cpu),
            Instruction::SWAP  { op } => swap(op, cpu),
            Instruction::EXG   { op } => exg(op, cpu),
            // clear/set
            Instruction::CLR   { op } => clr(op, cpu),
            Instruction::Scc   { op } => scc(op, cpu),
            // other
            Instruction::NOP               => {},   // TODO: "does not begin execution until all pending bus cycles have completed."
            Instruction::Undefined{op}=> trap_illegal(op, cpu),
        }
    }
}

fn move_(op: u16, cpu: &mut CPU) {
    let s = get_op_size_alt(op, SIZE_SHIFT_MOVE);
    let am_src = AddressingMode::decode_op(op);
    // Every addressing mode can be used as the source, so no need to assert validity

    let xn_hi = get_op_xn_hi(op);
    let m_hi = get_op_m_hi(op);
    let am_dst = AddressingMode::decode(m_hi, xn_hi);

    let src_val = cpu.read_am(s, am_src);
    cpu.write_am(s, am_dst, src_val);

    cpu.reg.sr.modify_n(is_negative(src_val, s));
    cpu.reg.sr.modify_z(is_zero(src_val, s));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}

fn movea(op: u16, cpu: &mut CPU) {
    let s = get_op_size_alt(op, SIZE_SHIFT_MOVE);
    let am_src = AddressingMode::decode_op(op);
    // Same as move; no need to assert validity of source

    let an = get_op_xn_hi(op).into();

    let val = cpu.read_am(s, am_src);
    let src_val = sign_extend_size(val, s);

    cpu.reg.write_address(an, DataSize::Long, src_val);
}

fn moveq(op: u16, cpu: &mut CPU) {
    let dn = get_op_xn_hi(op).into();
    let val = (op as u8).sign_extend();

    cpu.reg.write_data(dn, DataSize::Long, val);

    cpu.reg.sr.modify_n(is_negative(val, DataSize::Long));
    cpu.reg.sr.modify_z(is_zero(val, DataSize::Long));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}

fn movem(op: u16, cpu: &mut CPU) {
    let size = get_op_size_bit6(op);
    let am = AddressingMode::decode_op(op);
    let d = op & 0x0400 != 0;

    let list_mask = cpu.read_pc_inc_word();    // +4(1/0)
    let n = list_mask.count_ones();
    let (reversed, list_ptr, list_ptr_mod) = if let AddressingMode::AddrPreDec(_) = am {
        (true, 16, -1)
    } else {
        (false, 0, 1)
    };

    if d {
        // mem -> reg
        for i in 0..16 {
            if list_mask & (1 << i) != 0 {
                let w = match am {
                    AddressingMode::AddrPostInc(an) => {
                        cpu.read_am(size, am);
                        todo!();
                    }
                    _ => {
                        cpu.read_am(size, am);
                        todo!();
                    }
                };
                let val = sign_extend_size(w, size);

                let reg_num = (if reversed {7 - i & 0b111} else {i & 0b111}).into();
                let addr_reg = if reversed { i & 0b1000 == 0 } else { i & 0b1000 != 0};

                if addr_reg {
                    // An
                    cpu.reg.write_address(reg_num, size, val);
                } else {
                    // Dn
                    cpu.reg.write_data(reg_num, size, val);
                }
            }
        }
    } else {
        // reg -> mem
        todo!();
    }
}

fn movep(op: u16, cpu: &mut CPU) {
    let displacement = cpu.read_pc_inc_word();

    let an: RegisterNum = get_op_xn_lo(op).into();
    let dn: RegisterNum = get_op_xn_hi(op).into();

    let m_hi = get_op_m_hi(op);
    let direction = m_hi & 0b010 != 0;
    let size = get_op_size_bit6(op);

    // TODO: check if this is correct
    if direction {
        // reg -> mem
        // println!("MOVEP REG -> MEM");
        let val = cpu.reg.read_data(dn);

        let a_r = cpu.reg.read_address(an);
        let addr = a_r + displacement as u32;

        // CURR CYCLES: 8(2/0)
        match size {
            DataSize::Word => {
                cpu.write_mem_byte(addr, (val >> 8) as u8);  // +4(0/1) => 12(2/1)
                cpu.write_mem_byte(addr+2, val as u8);      // +4(0/1) => 16(2/2)
            },
            DataSize::Long => {
                cpu.write_mem_byte(addr, (val >> 24) as u8);           // +4(0/1) => 12(2/1)
                cpu.write_mem_byte(addr+2, (val >> 16) as u8);   // +4(0/1) => 16(2/2)
                cpu.write_mem_byte(addr+4, (val >> 8) as u8);    // +4(0/1) => 20(2/3)
                cpu.write_mem_byte(addr+6, val as u8);                // +4(0/1) => 24(2/4)
            },
            _ => unreachable!(),
        }
    } else {
        // mem -> reg
        // println!("MOVEP MEM -> REG");
        let a_r = cpu.reg.read_address(an);
        let addr = a_r + displacement as u32;

        let loops = match size {
            DataSize::Word => 2,
            DataSize::Long => 4,
            _ => unreachable!()
        };

        // CURR CYCLES: 8(2/0)
        for i in 0..loops {
            // +4(1/0)*loops => 12(3/0) => 16(4/0) [ => 20(5/0) => 24(6/0)]
            let val = cpu.read_mem_byte(addr + i * 2);
            cpu.reg.write_data_byte_offset(dn, (loops - 1) - i, val);
        }
    }
}

fn move_from_sr(op: u16, cpu: &mut CPU) {
    let am = AddressingMode::decode_op(op);

    if cpu.reg.sr.supervisor_state() || cpu.revision == CPURevision::MC68000 {
        let val = cpu.reg.sr.sr();
        // SR -> Destination
        cpu.write_am(DataSize::Word, am, val as u32);   // +x(r/w)

        if let AddressingMode::DataReg(_) = am {
            // move.w dn,sr is 6 cycles
            cpu.cycle(2);
        }
    } else {
        // trap
        trap_vec(CPUVector::PrivelegeViolation as u8, cpu);
    }
}

fn move_to_ccr(op: u16, cpu: &mut CPU) {
    let am = AddressingMode::decode_op(op);

    // Source -> CCR
    let val = cpu.read_am(DataSize::Word, am);  // +x(r/0)
    cpu.reg.sr.modify_ccr(val as u16);
    cpu.cycle(8);       // +8(0/0)
}

fn move_to_sr(op: u16, cpu: &mut CPU) {
    let am = AddressingMode::decode_op(op);

    if cpu.reg.sr.supervisor_state() {
        let val = cpu.read_am(DataSize::Word, am);  // +x(r/0)
        // Source -> SR
        cpu.reg.sr.modify_sr(val as u16, cpu.revision);
        cpu.cycle(8);   // +8(0/0)
    } else {
        // trap
        trap_vec(CPUVector::PrivelegeViolation as u8, cpu);
    }
}

fn move_usp(op: u16, cpu: &mut CPU) {
    if cpu.reg.sr.supervisor_state() {
        let an: RegisterNum = get_op_xn_lo(op).into();

        if op & 0b1000 == 0 {
            // an -> usp
            cpu.reg.usp = cpu.reg.read_address(an);
        } else {
            // usp -> an
            cpu.reg.write_address(an, DataSize::Long, cpu.reg.usp);
        }
    } else {
        // trap
        trap_vec(CPUVector::PrivelegeViolation as u8, cpu);
    }
}

fn set_sr_flags_add(cpu: &mut CPU, s: DataSize, old: u32, val: u32, res: u32) {
    let a = old & s.mask();
    let b = val & s.mask();
    let r = res & s.mask();

    let n = r & s.sign_bit() != 0;
    let z = r == 0;
    let v = (!(a ^ b) & (a ^ r) & s.sign_bit()) != 0;
    let c = r < a;

    cpu.reg.sr.modify_x(c);
    cpu.reg.sr.modify_n(n);
    cpu.reg.sr.modify_z(z);
    cpu.reg.sr.modify_v(v);
    cpu.reg.sr.modify_c(c);
}

fn set_sr_flags_sub(cpu: &mut CPU, s: DataSize, old: u32, val: u32, res: u32) {
    let a = old & s.mask();
    let b = val & s.mask();
    let r = res & s.mask();

    let n = r & s.sign_bit() != 0;
    let z = r == 0;
    let v = ((a ^ b) & (a ^ r) & s.sign_bit()) != 0;
    let c = a < b;

    cpu.reg.sr.modify_x(c);
    cpu.reg.sr.modify_n(n);
    cpu.reg.sr.modify_z(z);
    cpu.reg.sr.modify_v(v);
    cpu.reg.sr.modify_c(c);
}

fn set_sr_flags_logical(cpu: &mut CPU, s: DataSize, _: u32, _: u32, res: u32) {
    let r = res & s.mask();

    let n = r & s.sign_bit() != 0;
    let z = r == 0;
    let v = false;
    let c = false;

    cpu.reg.sr.modify_n(n);
    cpu.reg.sr.modify_z(z);
    cpu.reg.sr.modify_v(v);
    cpu.reg.sr.modify_c(c);
}

// Since add[a|q|...] and sub[a|q|...] are almost the exact same
macro_rules! define_op {
    ($add:ident, $modify_am_add:ident, $set_sr_flags_add:ident, $is_eor:expr) => {
        fn $add(op: u16, cpu: &mut CPU) {
            let am = AddressingMode::decode_op(op);

            let m = get_op_m_hi(op);
            let s = (m & 0b11).into();
            let dn = get_op_xn_hi(op).into();

            if m & 0b100 == 0 && !$is_eor {
                if s == DataSize::Long {
                    match am {
                        AddressingMode::DataReg(_) | AddressingMode::AddrReg(_) => cpu.cycle(4),
                        _ => cpu.cycle(2),
                    }
                }

                let val = cpu.read_am(s, am);

                // <ea> ' Dn -> Dn
                let (old, res) = cpu.$modify_am_add(s, AddressingMode::DataReg(dn), val);

                $set_sr_flags_add(cpu, s, old, val, res);
            } else {
                // TODO?: Is it possible for a Dy ' Dx -> Dx (where second <ea> = Dx) to be
                // decoded into this path (i.e. m & 0b100 != 0) instead of the above one?
                if s != DataSize::Long {
                    cpu.cycle(4);   // +4(0/0) (?: should be write cycle?)
                } else {
                    cpu.cycle(8);   // +8(0/0) (?: should be write cycles?)
                }

                let val = cpu.reg.read_data(dn);

                // Dn ' <ea> -> <ea>
                let (old, res) = cpu.$modify_am_add(s, am, val);

                $set_sr_flags_add(cpu, s, old, val, res);
            }
        }
    };
}

macro_rules! define_opa {
    ($adda:ident, $modify_am_add:ident, $set_sr_flags_add:ident) => {
        fn $adda(op: u16, cpu: &mut CPU) {
            let am = AddressingMode::decode_op(op);

            let s = get_op_size_bit8(op);
            let an = get_op_xn_hi(op).into();

            match am {
                AddressingMode::DataReg(_) | AddressingMode::AddrReg(_) => cpu.cycle(4),    // +4(0/0)
                AddressingMode::Imm if s == DataSize::Long => cpu.cycle(4),                 // +4(0/0)
                _ => if s != DataSize::Long { cpu.cycle(4) } else { cpu.cycle(2) },         // +4(0/0) | +2(0/0)
            }

            let val = cpu.read_am(s, am);

            // <ea> ' An -> An
            cpu.$modify_am_add(s, AddressingMode::AddrReg(an), val);
        }
    };
}

macro_rules! define_opi {
    ($addi:ident, $modify_am_add:ident, $set_sr_flags_add:ident) => {
        fn $addi(op: u16, cpu: &mut CPU) {
            let am = AddressingMode::decode_op(op);
            let s = get_op_size(op);

            let val = cpu.read_pc_inc_size(s);  // +4(1/0) | +8(2/0)

            match am {
                AddressingMode::DataReg(_) => {},
                _ => cpu.cycle(4),  // +4(0/0)  //? should be write?
            }

            if s == DataSize::Long {
                cpu.cycle(4);       // +4(0/0)  //? should be writes?
            }

            // <ea> ' <ea> -> <ea>
            let (old, res) = cpu.$modify_am_add(s, am, val);

            $set_sr_flags_add(cpu, s, old, val, res);
        }
    };
}

macro_rules! define_opq {
    ($addq:ident, $modify_am_add:ident, $set_sr_flags_add:ident) => {
        fn $addq(op: u16, cpu: &mut CPU) {
            let s = get_op_size(op);
            let am = AddressingMode::decode_op(op);

            let val = ((op & 0xe00) >> 9) as u32;

            match am {
                AddressingMode::DataReg(_) => {},
                AddressingMode::AddrReg(_) => if s == DataSize::Word { cpu.cycle(4) },  // +4(0/0)
                _ => cpu.cycle(4),  // +4(0/0)
            }

            if s == DataSize::Long {
                cpu.cycle(4);   // +4(0/0)
            }

            let (old, res) = cpu.$modify_am_add(s, am, val);

            $set_sr_flags_add(cpu, s, old, val, res);
        }
    };
}

macro_rules! define_opx {
    ($addx:ident, $modify_am_add:ident, $set_sr_flags_add:ident) => {
        fn $addx(op: u16, cpu: &mut CPU) {
            let xn_src = get_op_xn_lo(op).into();
            let xn_dst = get_op_xn_hi(op).into();
            let s = get_op_size(op);
            let m = op & 0b1000 == 0;

            if s == DataSize::Long {
                cpu.cycle(4);   // +4(0/0)
            }

            if m {
                // Dn
                let val = cpu.reg.read_data(xn_src);
                let x = cpu.reg.sr.x() as u32;

                // Dy ' Dx + X -> Dx
                let (old, res) = cpu.$modify_am_add(s, AddressingMode::DataReg(xn_dst), val + x);

                $set_sr_flags_add(cpu, s, old, val + x, res);
            } else {
                // -(An)
                let val = cpu.read_am(s, AddressingMode::AddrPreDec(xn_src));   // +6(1/0) | +10(2/0)
                let x = cpu.reg.sr.x() as u32;
                cpu.cycle(2);   // +2(0/0)

                // -(An) ' -(An) + X -> -(An)
                let (old, res) = cpu.$modify_am_add(s, AddressingMode::AddrPreDec(xn_dst), val + x); // + 6(1/0) | +10(2/0)

                $set_sr_flags_add(cpu, s, old, val + x, res);
            }
        }
    };
}

macro_rules! define_opi_to {
    (ccr, $fn_name:ident, $op:tt) => {
        fn $fn_name(cpu: &mut CPU) {
            let src = cpu.read_pc_inc_word() & 0xff;   // +4(1/0)
            cpu.read_pc_word();    // +4(1/0)  (dummy read)
            cpu.cycle(8);           // +8(0/0)
            // Source ' CCR -> CCR
            let dst = cpu.reg.sr.ccr() as u16;
            cpu.reg.sr.modify_ccr(src $op dst);
        }
    };

    (sr, $fn_name:ident, $op:tt) => {
        fn $fn_name(cpu: &mut CPU) {
            if cpu.reg.sr.supervisor_state() {
                let src = cpu.read_pc_inc_word();   // +4(1/0)
                cpu.read_pc_word();    // +4(1/0)  (dummy read)
                cpu.cycle(8);           // +8(0/0)
                // Source ' SR -> SR
                let dst = cpu.reg.sr.sr();
                cpu.reg.sr.modify_sr(src $op dst, cpu.revision);
            } else {
                trap_vec(CPUVector::PrivelegeViolation as u8, cpu);
            }
        }
    };
}

define_op! (add,  modify_am_add, set_sr_flags_add, false);
define_opa!(adda, modify_am_add, set_sr_flags_add);
define_opi!(addi, modify_am_add, set_sr_flags_add);
define_opq!(addq, modify_am_add, set_sr_flags_add);
define_opx!(addx, modify_am_add, set_sr_flags_add);

define_op! (sub,  modify_am_sub, set_sr_flags_sub, false);
define_opa!(suba, modify_am_sub, set_sr_flags_sub);
define_opi!(subi, modify_am_sub, set_sr_flags_sub);
define_opq!(subq, modify_am_sub, set_sr_flags_sub);
define_opx!(subx, modify_am_sub, set_sr_flags_sub);

fn div(op: u16, cpu: &mut CPU) {
    // TODO: the other mc680x0 (x = 2,3,4 or CPU32) div instructions

    let dn = get_op_xn_hi(op).into();
    let am = AddressingMode::decode_op(op);

    let src = cpu.read_am(DataSize::Word, am);
    let dst = cpu.reg.read_data(dn);

    let unsigned = op & 0b100_000_000 == 0;

    // A divide by zero exception supposedly takes 42(5/3) cycles (ignoring am..?), so we need 8 cycles before checking.
    cpu.cycle(8);   // +8(0/0)

    // "Always cleared", so even when div by 0..?
    cpu.reg.sr.clear_c();

    if src == 0 {
        return trap_vec(CPUVector::IntDivByZero as u8, cpu);
    }

    let (rem, quot) = if unsigned {
        // divu
        let d = dst as u32;
        let s = src as u32;

        let quot = d / s;
        let rem  = d % s;

        (rem as u32, quot as u32)
    } else {
        // divs
        // TODO: this always assumes the worst-case scenario. Check what the actual cycle count should be.
        // "The divide algorithm used by the MC68000 provides less than 10% difference between
        // the best and the worst case timings."
        cpu.cycle(18);  // +18(0/0)

        let d = dst as i32;
        let s = src as i16 as i32;

        let quot = d / s;
        let rem  = d % s;

        (rem as u32, quot as u32)
    };

    let v = if unsigned {
        quot > 0xffff
    } else {
        (quot as i32 > i16::MAX as i32) || ((quot as i32) < i16::MIN as i32)
    };

    if v {
        // "If the instruction detects an overflow, it sets the overflow condition code,
        // and the operands are unaffected."
        // TODO: check if the cycle counts when overflow are correct.
        cpu.reg.sr.set_v();
    } else {
        cpu.cycle(126);

        let res = ((rem & 0xffff) << 16) | (quot & 0xffff);

        cpu.reg.write_data(dn, DataSize::Long, res);

        cpu.reg.sr.modify_n(quot & DataSize::Word.sign_bit() != 0);
        cpu.reg.sr.modify_z(quot == 0);
    }

    // ?
    cpu.cycle(2);
}

fn mul(op: u16, cpu: &mut CPU) {
    // TODO: the other mc680x0 (x = 2,3,4 or CPU32) mul instructions

    let dn = get_op_xn_hi(op).into();
    let am = AddressingMode::decode_op(op);

    let src = cpu.read_am(DataSize::Word, am);
    let dst = cpu.reg.read_data(dn) & 0xffff;

    let unsigned = op & 0b100_000_000 == 0;

    // "The multiply algorithm requires 38+2n clocks where"
    let n = if unsigned {
        // "n = the number of ones in the <ea>"
        src.count_ones()
    } else {
        // "n = concatenate the <ea> with a zero as the LSB;
        //    n is the resultant number of 10 or 01 patterns
		//    in the 17-bit source; i.e., worst case happens
		//    when the source is $5555"
        let mut src_s = src << 1;
        let mut i = 0;
        for _ in 0..17 {
            if src_s & 0b11 == 0b10 || src_s & 0b11 == 0b01 {
                i += 1;
            }
            src_s >>= 1;
        }
        // ?
        assert!(i <= 16);
        i
    };
    cpu.cycle(32 + 2 * (n as usize));

    let res = if unsigned {
        // mulu
        src * dst
    } else {
        // muls
        (src as i16 as i32 * dst as i16 as i32) as u32
    };

    cpu.reg.write_data(dn, DataSize::Long, res);

    cpu.reg.sr.modify_n(res & DataSize::Long.sign_bit() != 0);
    cpu.reg.sr.modify_z(res == 0);
    // TODO: "Overflow (V = 1) can occur only when multiplying 32-bit operands to yield a 32-bit result."
    cpu.reg.sr.clear_c();

    // ?
    cpu.cycle(2);
}

define_op! (or,   modify_am_or,  set_sr_flags_logical, false);
define_opi!(ori,  modify_am_or,  set_sr_flags_logical);

define_op! (and,  modify_am_and, set_sr_flags_logical, false);
define_opi!(andi, modify_am_and, set_sr_flags_logical);

define_op! (eor,  modify_am_eor, set_sr_flags_logical, true);
define_opi!(eori, modify_am_eor, set_sr_flags_logical);

define_opi_to!(ccr, ori_to_ccr,  |);
define_opi_to!(sr,  ori_to_sr,   |);
define_opi_to!(ccr, andi_to_ccr, &);
define_opi_to!(sr,  andi_to_sr,  &);
define_opi_to!(ccr, eori_to_ccr, ^);
define_opi_to!(sr,  eori_to_sr,  ^);

fn cmp(op: u16, cpu: &mut CPU) {
    let s = get_op_size(op);
    let am = AddressingMode::decode_op(op);
    // (all modes are valid)
    let dn = get_op_xn_hi(op).into();

    if s == DataSize::Long {
        cpu.cycle(2);   // +2(0/0)
    }

    let src = cpu.read_am(s, am);
    let dst = cpu.reg.read_data(dn);
    let res = dst - src;

    // x is modified in `sub`, but NOT in `cmp`
    let old_x = cpu.reg.sr.x(); 
    set_sr_flags_sub(cpu, s, dst, src, res);
    cpu.reg.sr.modify_x(old_x);
}

fn cmpa(op: u16, cpu: &mut CPU) {
    let s = get_op_size_bit8(op);
    let am = AddressingMode::decode_op(op);
    // (all modes are valid)
    let an = get_op_xn_hi(op).into();

    cpu.cycle(2);   // +2(0/0)

    let src_pre = cpu.read_am(s, am);
    let src = sign_extend_size(src_pre, s);

    let dst = cpu.reg.read_address(an);

    let res = dst - src;

    // x is modified in `sub`, but NOT in `cmp`
    let old_x = cpu.reg.sr.x(); 
    set_sr_flags_sub(cpu, s, dst, src, res);
    cpu.reg.sr.modify_x(old_x);
}

fn cmpm(op: u16, cpu: &mut CPU) {
    let s = get_op_size(op);
    let ay = get_op_xn_lo(op).into();
    let ax = get_op_xn_hi(op).into();

    let src = cpu.read_am(s, AddressingMode::AddrPostInc(ax));
    let dst = cpu.read_am(s, AddressingMode::AddrPostInc(ay));
    let res = dst - src;

    // x is modified in `sub`, but NOT in `cmp`
    let old_x = cpu.reg.sr.x(); 
    set_sr_flags_sub(cpu, s, dst, src, res);
    cpu.reg.sr.modify_x(old_x);
}

fn cmpi(op: u16, cpu: &mut CPU) {
    let s = get_op_size(op);
    let am = AddressingMode::decode_op(op);

    if let AddressingMode::DataReg(_) = am && s == DataSize::Long {
        cpu.cycle(2);   // +2(0/0)
    }

    let src = cpu.read_pc_inc_size(s);
    let dst = cpu.read_am(s, am);
    let res = dst - src;

    // x is modified in `sub`, but NOT in `cmp`
    let old_x = cpu.reg.sr.x(); 
    set_sr_flags_sub(cpu, s, dst, src, res);
    cpu.reg.sr.modify_x(old_x);
}

fn neg(op: u16, cpu: &mut CPU) {
    let s = get_op_size(op);
    let am = AddressingMode::decode_op(op);

    match am {
        AddressingMode::DataReg(_) => if s == DataSize::Long { cpu.cycle(2) },      // +0(0/0) | +2(0/0)
        _ => if s == DataSize::Long { cpu.cycle(8) } else { cpu.cycle(4) }, // +8(0/0) (?: should be write(s)?)
    }

    let negx = (((op & 0b100_00_000_000) >> 10) ^ 1) as u32;
    let (old, res) = cpu.modify_am_neg(s, am, negx);

    let a = 0;
    let b = old & s.mask();
    let r = res & s.mask();

    let z = r == 0;
    let n = r & s.sign_bit() != 0;
    let v = ((a ^ b) & (a ^ r) & s.sign_bit()) != 0;
    let c = !z;

    cpu.reg.sr.modify_x(c);
    cpu.reg.sr.modify_n(n);
    cpu.reg.sr.modify_z(z);
    cpu.reg.sr.modify_v(v);
    cpu.reg.sr.modify_c(c);
}

fn not(op: u16, cpu: &mut CPU) {
    let s = get_op_size(op);
    let am = AddressingMode::decode_op(op);

    match am {
        AddressingMode::DataReg(_) => if s == DataSize::Long { cpu.cycle(2) },      // +0(0/0) | +2(0/0)
        _ => if s == DataSize::Long { cpu.cycle(8) } else { cpu.cycle(4) }, // +8(0/0) (?: should be write(s)?)
    }

    let (_, res) = cpu.modify_am_not(s, am);

    cpu.reg.sr.modify_n(is_negative(res, s));
    cpu.reg.sr.modify_z(is_zero(res, s));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}

macro_rules! define_bop {
    ($fn_name: ident, $modify_am_fn: ident, $modify: expr) => {
        fn $fn_name(op: u16, cpu: &mut CPU) {
            let am = AddressingMode::decode_op(op);

            let xn = get_op_xn_hi(op);

            let (s, size_mask) = match am {
                AddressingMode::DataReg(_) => {
                    cpu.cycle(2);                           // +2(0/0)
                    (DataSize::Long, 0x1f)
                },
                _ => (DataSize::Byte, 0b111),
            };

            let bit = if xn != 0b100 {
                // dn,<ea>
                let dn = xn.into();
                cpu.reg.read_data(dn) & size_mask
            } else {
                // #<data>,<ea>
                cpu.read_pc_inc_word() as u32 & size_mask  // +4(1/0)
            };

            let old = if $modify {
                if s == DataSize::Byte {
                    cpu.cycle(4);
                }
                cpu.$modify_am_fn(s, am, bit).0
            } else {
                cpu.read_am(s, am)
            };

            cpu.reg.sr.modify_z(old & (1 << bit) == 0);
        }
    };
}

define_bop!(btst, modify_am_set, false);
define_bop!(bset, modify_am_set, true);
define_bop!(bclr, modify_am_clr, true);
define_bop!(bchg, modify_am_chg, true);

fn tas(op: u16, cpu: &mut CPU) {
    let am = AddressingMode::decode_op(op);

    let s = DataSize::Byte;
    let (val,_) = cpu.modify_am_tas(s, am);

    match am {
        AddressingMode::DataReg(_) => {},
        _ => cpu.cycle(6),  // +6(0/0)
    }

    cpu.reg.sr.modify_n(is_negative(val, s));
    cpu.reg.sr.modify_z(is_zero(val, s));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();

    // "The operation uses a locked or read-modify-write transfer sequence.
    // This instruction supports use of a ﬂag or semaphore to coordinate several processors."
    // TODO: handle the above (on different cpu revisions?)
}

fn tst(op: u16, cpu: &mut CPU) {
    let s = get_op_size(op);
    let am = AddressingMode::decode_op(op);

    let val = cpu.read_am(s, am);

    cpu.reg.sr.modify_n(is_negative(val, s));
    cpu.reg.sr.modify_z(is_zero(val, s));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}

macro_rules! define_shift_rot_m {
    ($fn_name:ident, $shift_op:ident, $modify_x:expr, $modify_v:expr) => {
        fn $fn_name(op: u16, cpu: &mut CPU) {
            // Shift memory. Always shifts a single bit and data size is always a word.
            let d = (op & 0b100_000_000) >> 8 != 0;
            let am = AddressingMode::decode_op(op);

            let s = DataSize::Word;

            cpu.cycle(4);   // +4(0/0)  (?: should be write?)

            // Let's just do the am handling manually instead of adding more modify_am_* fn's
            let am_addr = cpu.calculate_am_operand(s, am);
            let mut mem_val = cpu.read_mem_size_internal(s, am_addr);

            let c;

            let prev_sign_bit = mem_val & s.sign_bit();

            if d {
                $shift_op!(l, c, mem_val , s);
            } else {
                $shift_op!(r, c, mem_val , s);
            }

            let msb_changed = ((mem_val & s.sign_bit()) ^ prev_sign_bit) != 0;

            if $modify_x {
                cpu.reg.sr.modify_x(c);
            }
            cpu.reg.sr.modify_n(mem_val & s.sign_bit() != 0);
            cpu.reg.sr.modify_z(mem_val == 0);
            if $modify_v {
                cpu.reg.sr.modify_v(msb_changed);
            }
            cpu.reg.sr.modify_c(c);

            cpu.write_mem_word(am_addr, mem_val as u16);
        }
    };
}

macro_rules! define_shift_rot_d {
    ($fn_name:ident, $shift_op:ident, $modify_x:expr, $modify_v:expr) => {
        fn $fn_name(op: u16, cpu: &mut CPU) {
            // Shift data reg
            let m = (op & 0b100_000) != 0;
            let s = get_op_size(op);
            let d = (op & 0b100_000_000) != 0;

            if s == DataSize::Long {
                cpu.cycle(4);   // +4(0/0)
            } else {
                cpu.cycle(2);   // +2(0/0)
            }

            let dn_to_shift = get_op_xn_lo(op).into();
            let count = get_op_xn_hi(op);

            let shift_count = if m {
                // Dx,Dy   ; Dy << Dx
                cpu.reg.read_data(count.into()) & 0x3f
            } else {
                // #imm,Dy ; Dy << #imm
                if count == 0 {
                    8
                } else {
                    count as u32 & 0x7
                }
            };

            let mut dn_val = cpu.reg.read_data(dn_to_shift);
            let mut c = false;
            let mut msb_changed = false;

            for _ in 0..shift_count {
                let prev_sign_bit = dn_val & s.sign_bit();
                if d {
                    $shift_op!(l, c, dn_val, s);
                } else {
                    $shift_op!(r, c, dn_val, s);
                }

                if !msb_changed {
                    msb_changed = ((dn_val & s.sign_bit()) ^ prev_sign_bit) != 0;
                }

                if $modify_x {
                    cpu.reg.sr.modify_x(c);
                }
                // cpu.reg.sr.modify_c(c);

                //? Is this needed every shift?
                cpu.reg.write_data(dn_to_shift, s, dn_val);
                cpu.cycle(2);
            }

            //? Should these change during the shifting too?
            cpu.reg.sr.modify_n(dn_val & s.sign_bit() != 0);
            cpu.reg.sr.modify_z(dn_val == 0);
            if $modify_v {
                cpu.reg.sr.modify_v(msb_changed);
            }
            cpu.reg.sr.modify_c(c);
        }

    };
}

macro_rules! asd_op {
    (l, $c:expr, $dn_val:expr, $s:expr) => { 
        // asl
        $c = ($dn_val & $s.sign_bit()) != 0;
        $dn_val = ($dn_val << 1) & $s.mask();
    };
    (r, $c:expr, $dn_val:expr, $s:expr) => {
        // asr
        $c = ($dn_val & 1) != 0;
        let msb = $dn_val & $s.sign_bit();
        $dn_val = (($dn_val & $s.mask()) >> 1) | msb;
    };
}

macro_rules! lsd_op {
    (l, $c:expr, $dn_val:expr, $s:expr) => { 
        // lsl
        $c = ($dn_val & $s.sign_bit()) != 0;
        $dn_val = ($dn_val << 1) & $s.mask();
    };
    (r, $c:expr, $dn_val:expr, $s:expr) => {
        // lsr
        $c = ($dn_val & 1) != 0;
        $dn_val = ($dn_val & $s.mask()) >> 1;
    };
}

macro_rules! rod_op {
    (l, $c:expr, $dn_val:expr, $s:expr) => { 
        // rol
        $c = ($dn_val & $s.sign_bit()) != 0;
        $dn_val = (($dn_val << 1) & $s.mask()) | ($c as u32);
    };
    (r, $c:expr, $dn_val:expr, $s:expr) => {
        // ror
        $c = ($dn_val & 1) != 0;
        $dn_val = (($dn_val & $s.mask()) >> 1) | (($c as u32) << $s.msb_shift());
    };
}

define_shift_rot_m!(asd_m,  asd_op, true,  true);
define_shift_rot_m!(lsd_m,  lsd_op, true,  false);
define_shift_rot_m!(roxd_m, rod_op, true,  false);
define_shift_rot_m!(rod_m,  rod_op, false, false);

define_shift_rot_d!(asd_d,  asd_op, true,  true);
define_shift_rot_d!(lsd_d,  lsd_op, true,  false);
define_shift_rot_d!(roxd_d, rod_op, true,  false);
define_shift_rot_d!(rod_d,  rod_op, false, false);

macro_rules! define_xbcd {
    ($abcd:ident, $modify_am_abcd:ident) => {
        fn $abcd(op: u16, cpu: &mut CPU) {
            let xx = get_op_xn_hi(op).into();
            let xy = get_op_xn_lo(op).into();
            let m = op & 0b1000 == 0;
            let s = DataSize::Byte;

            cpu.cycle(2);   // +2(0/0)

            let x = cpu.reg.sr.x();

            let (carry, res) = if m {
                // Dy,Dx
                let dy_val = cpu.reg.read_data(xy);

                // Dy ' Dx -> Dx
                cpu.$modify_am_abcd(s, AddressingMode::DataReg(xx), dy_val, x)
            } else {
                // -(Ay),-(Ax)
                let ay_val = cpu.read_am(s, AddressingMode::AddrPreDec(xy));        // +6(1/0)

                // -(An) ' -(An) -> -(An)
                cpu.$modify_am_abcd(s, AddressingMode::AddrPreDec(xx), ay_val, x)   // + 6(1/0)
            };

            cpu.reg.sr.modify_x(carry);
            // n is undefined
            if res != 0 {
                cpu.reg.sr.clear_z();
            }
            // v is undefined
            cpu.reg.sr.modify_c(carry);
        }

    };
}

define_xbcd!(abcd, modify_am_abcd);
define_xbcd!(sbcd, modify_am_sbcd);

fn nbcd(op: u16, cpu: &mut CPU) {
    let am = AddressingMode::decode_op(op);
    let s = DataSize::Byte;

    match am {
        AddressingMode::DataReg(_) => cpu.cycle(2), // +2(0/0)
        _ => cpu.cycle(4),                          // +4(0/0)
    }

    let x = cpu.reg.sr.x();
    let (borrow, res) = cpu.modify_am_nbcd(s, am, 0, x);

    cpu.reg.sr.modify_x(borrow);
    // n is undefined
    if res != 0 {
        cpu.reg.sr.clear_z();
    }
    // v is undefined
    cpu.reg.sr.modify_c(borrow);
}

fn lea(op: u16, cpu: &mut CPU) {
    let am = AddressingMode::decode_op(op);
    let an: RegisterNum = get_op_xn_hi(op).into();

    let val = cpu.calculate_am_operand(DataSize::Long, am);
    // <ea> -> An
    cpu.reg.write_address(an, DataSize::Long, val);
}

fn pea(op: u16, cpu: &mut CPU) {
    let am = AddressingMode::decode_op(op);

    let val = cpu.calculate_am_operand(DataSize::Long, am);
    // SP - 4 -> SP;
    // <ea> -> (SP)
    cpu.push_sp_long(val);
}

/// Cycles: 30(3/3) (MC68000)
fn trap_vec(vector_num: u8, cpu: &mut CPU) {
    let prev_sr = cpu.reg.sr.sr();

    cpu.reg.sr.set_supervisor();

    // ?
    cpu.read_pc_word();    // +4(1/0)

    if cpu.revision != CPURevision::MC68000 {
        // As is written elsewhere:
        // "The MC68000 and MC68008 do not write vector offset or
        // format code to the system stack"
        // Format + Offset word -> (SSP) 
        // SSP - 2 -> SSP
        let format_offset = 0x0000_0000;
        cpu.push_sp_word(format_offset);  // +4(0/1) => 4(0/1)
        cpu.cycle(2);   // ?
        todo!("Format/offset -> (SSP)")
    }

    // PC -> (SSP)
    // SSP - 4 -> SSP
    cpu.push_sp_long(cpu.reg.pc);        // +8(0/2) => ??(1/2)
    cpu.cycle(2);   // ?

    // SR -> (SSP)
    // SSP - 2 -> SSP
    cpu.push_sp_word(prev_sr);          // +4(0/1) => ??(1/3)
    cpu.cycle(2);   // ?

    // (Vector) -> PC
    cpu.reg.pc = cpu.read_mem_long(vector_num as u32 * CPU_VECTOR_SIZE);// +8(2/0) => 24(3/3)  (or 28(3/4))
    cpu.cycle(2);
}

fn trapv(cpu: &mut CPU) {
    if cpu.reg.sr.v() {
        trap_vec(CPUVector::TRAPV as u8, cpu);   // +30(3/3) => 34(4/3) (4(1/0) when v == 0)
    }
}

fn trap(op: u16, cpu: &mut CPU) {
    // ? There seems to be conflicting information as to whether this instruction should
    // ? take 34 or 38 cycles. For now, let's assume it does.
    cpu.cycle(4);   // + 4(0/0)

    let v = (op & 0xf) as u8;
    trap_vec(v + 32, cpu);
}

fn trap_illegal(_op: u16, cpu: &mut CPU) {
    #[cfg(feature = "std")]
    println!("illegal instruction (0b{_op:016b}; 0x{_op:04x})");

    trap_vec(CPUVector::IllegalInstruction as u8, cpu);
}

fn chk(op: u16, cpu: &mut CPU) {
    let s = get_op_size_alt(op, SIZE_SHIFT_CHK);
    let am = AddressingMode::decode_op(op);
    let dn = get_op_xn_hi(op).into();

    cpu.cycle(6);       // +6(0/0)
    let dn_val = cpu.reg.read_data(dn);
    let src_val = cpu.read_am(s, am);

    let lt_cond = dn_val & s.sign_bit() != 0;
    let gt_cond = dn_val & s.mask() > src_val;

    if lt_cond || gt_cond {
        // ?
        cpu.read_pc_word();   // +4(1/0)

        //? N is "Set if Dn < 0; cleared if Dn > effective address operand; undefined otherwise."
        if lt_cond {
            cpu.reg.sr.clear_n();
        } else if gt_cond {
            cpu.reg.sr.clear_n();
        }

        trap_vec(CPUVector::ChkInstruction as u8, cpu);
    }

    //? Note: the rest of the condition codes are "undefined" (except x, which is "not affected")
    //? ...should this be handled in a specific way?
}

fn stop(cpu: &mut CPU) {
    if cpu.reg.sr.supervisor_state() {
        let imm_data = cpu.read_pc_inc_word(); // +4(1/0)
        cpu.reg.sr.modify_sr(imm_data, cpu.revision);
        cpu.stopped = true;
    } else {
        trap_vec(CPUVector::PrivelegeViolation as u8, cpu);
    }
}

fn reset(cpu: &mut CPU) {
    if cpu.reg.sr.supervisor_state() {
        let assert_cycles = match cpu.revision {
            CPURevision::MC68000 | CPURevision::MC68010 => 124,
            _ => 512,
        };

        // Total cycles incl. opcode fetch should be 132 (on mc68000).
        //? Should this happen before or after the other cycles?
        cpu.cycle(4);

        cpu.reset_signal = true;
        cpu.cycle(assert_cycles);   // +124(0/0) / +512(0/0)
        cpu.reset_signal = false;
    } else {
        trap_vec(CPUVector::PrivelegeViolation as u8, cpu);
    }
}

fn jmp(op: u16, cpu: &mut CPU) {
    let am = AddressingMode::decode_op(op);

    // jumps work a bit different from usual instructions in that working with
    // "pointers" leads to jumping to the pointer, not reading what's at that address.
    // So, we simply skip the middle-man (`cpu.read_am`) and calculate directly.
    // TODO: also check that this matches up with cycle counts
    let destination_addr = cpu.calculate_am_operand(DataSize::Long, am);

    if op & 0b1_000_000 == 0 {
        // jsr
        // SP - 4 -> SP;
        // PC -> (SP);
        // TODO?: should this happen before or after am operand calculation?
        cpu.push_sp_long(cpu.reg.pc);  // + 8(0/2)
    }

    // Destination Address -> PC
    cpu.reg.pc = destination_addr;
}

/// Get the displacement operand (curr byte or next word/long) following
/// the current instruction. Does _not_ increase `pc`.
///
/// Cycles: .b, .w: +4(1/0) | .l: +8(2/0)
fn get_displacement_no_inc(cpu: &mut CPU, disp_tmp: u16) -> i32 {
    if disp_tmp == 0x00 { 
        cpu.read_pc_word().sign_extend() as i32  // +4(1/0)
    } else if disp_tmp == 0xff
        && (cpu.revision == CPURevision::MC68020
        ||  cpu.revision == CPURevision::MC68030
        ||  cpu.revision == CPURevision::MC68040)
    {
        cpu.read_pc_long() as i32          // +8(2/0)
    } else {
        // Seems to have the same amount of cycles as 16 bit displacement,
        // so we just read at the pc anyway.
        cpu.read_pc_word();            // +4(1/0)
        (disp_tmp as u8).sign_extend() as i32
    }
}

fn bra(op: u16, cpu: &mut CPU) {
    let disp_tmp = op & 0x00ff;
    let displacement = get_displacement_no_inc(cpu, disp_tmp); // .b, .w: +4(1/0) | .l: +8(2/0)

    // PC + d_n -> PC
    cpu.reg.pc = cpu.reg.pc.wrapping_add_signed(displacement);
    cpu.cycle(2);
}

fn bsr(op: u16, cpu: &mut CPU) {
    let disp_tmp = op & 0x00ff;
    let displacement = get_displacement_no_inc(cpu, disp_tmp); // .b, .w: +4(1/0) | .l: +8(2/0)

    // SP - 4 -> SP;
    // PC -> (SP);
    cpu.push_sp_long(cpu.reg.pc);   // +8(0/2)

    // PC + d_n -> PC
    cpu.reg.pc = cpu.reg.pc.wrapping_add_signed(displacement);
    cpu.cycle(2);
}

fn bcc(op: u16, cpu: &mut CPU) {
    let disp_tmp = op & 0x00ff;
    let condition = (op & 0x0f00) >> 8;
    let condition: Condition = condition.into();

    if condition.is_true(cpu) {
        let displacement = get_displacement_no_inc(cpu, disp_tmp); // .b, .w: +4(1/0) | .l: +8(2/0)

        // PC + d_n -> PC
        cpu.reg.pc = cpu.reg.pc.wrapping_add_signed(displacement);
        cpu.cycle(2);       // +2(0/0)
    } else {
        cpu.cycle(4);       // +4(0/0)

        if disp_tmp == 0x00 {
            // .w has 4 more cycles when branch not taken...
            cpu.cycle(4);   // +4(0/0)
        }
    }
}

fn dbcc(op: u16, cpu: &mut CPU) {
    // DBcc
    let condition = (op & 0x0f00) >> 8;
    let condition: Condition = condition.into();
    let disp = cpu.read_pc_word();

    // CURR CYCLES: 8(2/0)
    if !condition.is_true(cpu) {
        let dn: RegisterNum = get_op_xn_lo(op).into();
        // Dn - 1 -> Dn;
        let dn_val = cpu.reg.read_data(dn);
        cpu.reg.write_data(dn, DataSize::Word, dn_val - 1);

        if cpu.reg.read_data(dn) & 0xffff == 0xffff {
            // branch taken => cycs: 10(2/0)
            let displacement = disp.sign_extend() as i32;

            // PC + d_n -> PC
            cpu.reg.pc = cpu.reg.pc.wrapping_add_signed(displacement);
        } else {
            // branch not taken => cycs: 14(3/0)
            cpu.read_pc_inc_word();    // +4(1/0)
        }
        cpu.cycle(2);           // +2(0/0)
    } else {
        // cc is true => cycs: 12(2/0)
        cpu.reg.pc += 2;
        cpu.cycle(4);           // +4(0/0)
    }
}

fn rte(cpu: &mut CPU) {
    // All the rt[e|s|r] instructions seemingly have an extra read cycle:
    // - RTS: 16(4/0)
    // - RTR: 20(5/0)
    // - RTE: 20(5/0)
    cpu.read_pc_word();                     // +4(1/0) (dummy read)

    if cpu.reg.sr.supervisor_state() {
        // (SP) -> SR;
        // SP + 2 -> PC;
        let val = cpu.pop_sp_word();   // +4(1/0)
        cpu.reg.sr.modify_sr(val, cpu.revision);

        // (SP) -> PC;
        // SP + 4 -> SP;
        let old_state = cpu.pop_sp_long();  // +8(2/0)
        cpu.reg.pc = old_state;

        if cpu.revision != CPURevision::MC68000 {
            // Restore state and deallocate stack according to (SP)
            todo!("rte with {old_state:08x}");
        }
    } else {
        trap_vec(CPUVector::PrivelegeViolation as u8, cpu);
    }
}

fn rts(cpu: &mut CPU) {
    // Like rte and rtr, this incsruction has an extra read cycle
    cpu.read_pc_word();                     // +4(1/0) (dummy read)

    // (SP) -> PC;
    // SP + 4 -> SP
    cpu.reg.pc = cpu.pop_sp_long();          // +8(2/0)
}

fn rtr(cpu: &mut CPU) {
    // Like rts and rte, this incsruction has an extra read cycle
    cpu.read_pc_word();                     // +4(1/0) (dummy read)

    // (SP) -> CCR;
    // SP + 2 -> SP
    let val = cpu.pop_sp_word();
    cpu.reg.sr.modify_ccr(val);              // +4(1/0)
    // (SP) -> PC;
    // SP + 4 -> SP
    cpu.reg.pc = cpu.pop_sp_long();          // +8(2/0)
}

fn link(op: u16, cpu: &mut CPU) {
    let an: RegisterNum = get_op_xn_lo(op).into();

    let displacement = if op & 0b0000011001010000 != 0 {
        // word
        cpu.read_pc_inc_word().sign_extend() // +4(1/0)
    } else {
        // long (only on mc680x0 where x >= 2)
        cpu.read_pc_inc_long()  //
    };

    // SP - 4 -> SP;
    // An -> (SP);
    cpu.push_sp_long(cpu.reg.read_address(an)); // + 8(0/2)

    // SP -> An;
    let sp = cpu.reg.read_address(RegisterNum::R7);
    cpu.reg.write_address(an, DataSize::Long, sp);

    // SP + d_n -> SP
    cpu.reg.write_address(RegisterNum::R7, DataSize::Long, sp + displacement);
}

fn unlk(op: u16, cpu: &mut CPU) {
    let an: RegisterNum = get_op_xn_lo(op).into();

    let an_val = cpu.reg.read_address(an);
    // An -> SP;
    cpu.reg.write_address(RegisterNum::R7, DataSize::Long, an_val);

    // (SP) -> An;
    // SP + 4 -> SP
    let val = cpu.pop_sp_long();    // +8(2/0)
    cpu.reg.write_address(an, DataSize::Long, val);
}

fn ext(op: u16, cpu: &mut CPU) {
    let dn = get_op_xn_lo(op).into();
    let s = get_op_size_bit6(op);

    let val = cpu.reg.read_data(dn);
    let res = match s {
        DataSize::Byte => unreachable!(),
        DataSize::Word => sign_extend_size(val, DataSize::Byte) & 0x0000_ffff,
        DataSize::Long => sign_extend_size(val, DataSize::Word),
    };
    cpu.reg.write_data(dn, s, res);

    cpu.reg.sr.modify_n(is_negative(res, s));
    cpu.reg.sr.modify_z(is_zero(res, s));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}

fn swap(op: u16, cpu: &mut CPU) {
    let dn = get_op_xn_lo(op).into();

    let val = cpu.reg.read_data(dn);
    let val_hi = (val & 0xffff_0000) >> 8;
    let val_lo = val & 0x0000_ffff;
    let res = (val_lo << 8) | val_hi;

    let s = DataSize::Long;
    cpu.reg.write_data(dn, s, res);

    cpu.reg.sr.modify_n(is_negative(res, s));
    cpu.reg.sr.modify_z(is_zero(res, s));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}

fn exg(op: u16, cpu: &mut CPU) {
    let xy = get_op_xn_lo(op).into();
    let xx = get_op_xn_hi(op).into();
    let m_hi = (op & 0b11_000_000) >> 6;
    let m_lo = (op & 0b1_000) != 0;

    cpu.cycle(2);

    if m_hi == 0b01 && !m_lo {
        // Dx <-> Dy
        let dx_val = cpu.reg.read_data(xx);
        let dy_val = cpu.reg.read_data(xy);
        cpu.reg.write_data(xx, DataSize::Long, dy_val);
        cpu.reg.write_data(xy, DataSize::Long, dx_val);
    } else if m_hi == 0b01 {
        // Ax <-> Ay
        let ax_val = cpu.reg.read_address(xx);
        let ay_val = cpu.reg.read_address(xy);
        cpu.reg.write_address(xx, DataSize::Long, ay_val);
        cpu.reg.write_address(xy, DataSize::Long, ax_val);
    } else {
        // Dx <-> Ay
        let dx_val = cpu.reg.read_data(xx);
        let ay_val = cpu.reg.read_address(xy);
        cpu.reg.write_data(xx, DataSize::Long, ay_val);
        cpu.reg.write_address(xy, DataSize::Long, dx_val);
    }
}

fn clr(op: u16, cpu: &mut CPU) {
    let s = get_op_size(op);
    let am = AddressingMode::decode_op(op);

    match am {
        AddressingMode::DataReg(_) => if s == DataSize::Long { cpu.cycle(2) },      // +0(0/0) | +2(0/0)
        _ => if s == DataSize::Long { cpu.cycle(8) } else { cpu.cycle(4) }, // +8(0/0) (?: should be write(s)?)
    }

    if cpu.revision == MC68000 {
        // "In the MC68000 and MC68008 a memory location is read before it is cleared."
        cpu.modify_am_clr_all(s, am);
    } else {
        cpu.write_am(s, am, 0);
    }

    cpu.reg.sr.clear_n();
    cpu.reg.sr.set_z();
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}

fn scc(op: u16, cpu: &mut CPU) {
    let am = AddressingMode::decode_op(op);
    let condition = Condition::from((op & 0xf00) >> 8);

    match am {
        AddressingMode::DataReg(_) => {},
        _ => cpu.cycle(4),  // +4(0/0)
    }
    if condition.is_true(cpu) {
        // cc true

        if let AddressingMode::DataReg(_) = am {
            cpu.cycle(2);   // +2(0/0)
        }

        if cpu.revision == MC68000 {
            // "In the MC68000 and MC68008 a memory destination is read before it is written."
            cpu.modify_am_set_all(DataSize::Byte, am);
        } else {
            cpu.write_am(DataSize::Byte, am, 0xff);
        }
    } else {
        // cc false
        if cpu.revision == MC68000 {
            // "In the MC68000 and MC68008 a memory destination is read before it is written."
            cpu.modify_am_clr_all(DataSize::Byte, am);
        } else {
            cpu.write_am(DataSize::Byte, am, 0);
        }
    }
}
