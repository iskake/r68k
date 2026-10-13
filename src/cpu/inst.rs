use std::fmt::Display;

use crate::{bits::{DataSize, is_negative, is_zero, sign_extend, sign_extend_byte}, cpu::{CPU, RegisterNum}};

enum DataType {
    Immediate,
    BitIndex,
    Displacement,
    DisplacementOptional,
    RegisterListMask,
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
    /// Absolute short `(xxx).W`
    AbsShort,
    /// Absolute long `(xxx).L`
    AbsLong,
    /// Immediate `#imm`
    Imm,
    /// A mode that does not exist.
    Invalid,
}

impl Display for AddressingMode {
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
            AddressingMode::AbsShort                      => write!(f, "(xxx).w"),
            AddressingMode::AbsLong                       => write!(f, "(xxx).l"),
            AddressingMode::Imm                           => write!(f, "#imm"),
            AddressingMode::Invalid                       => write!(f, "(invalid addressing mode)"),
        }
    }
}

impl AddressingMode {
    fn decode(mode: u8, reg: u8) -> AddressingMode {
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
                    0b000 => AbsShort,
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

    fn decode_op(op: u16) -> AddressingMode {
        let mode = get_op_m_lo(op);
        let reg  = get_op_xn_lo(op);
        AddressingMode::decode(mode, reg)
    }
}

#[derive(Debug)]
enum VariationCCRSR {
    Normal,
    ToCCR,
    ToSR,
}

impl VariationCCRSR {
    fn from_op(op: u16) -> VariationCCRSR {
        match op & 0b11_111_111 {
            0b00_111_100 => VariationCCRSR::ToCCR,
            0b01_111_100 => VariationCCRSR::ToSR,
            _ => VariationCCRSR::Normal,
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    // 0000
    ORI   { op: u16 },
    ANDI  { op: u16 },
    SUBI  { op: u16 },
    ADDI  { op: u16 },
    EORI  { op: u16 },
    CMPI  { op: u16 },
    BTST  { op: u16 }, // == BTST, BCHG, BCLR, BSET
    MOVEP { op: u16 },
    // 00ss
    MOVEA { op: u16 },
    MOVE  { op: u16 },
    // 0100
    MOVEToFrom { op: u16 },
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
    ASd  { op: u16 },
    LSd  { op: u16 },
    ROXd { op: u16 },
    ROd  { op: u16 },
    // ASd, LSd, ROXd, ROd (rotation)
    // 1111
    // Floating point/supervisor instructions?

    // Extra
    Undefined { op: u16 },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;
        match self {
            ORI   { op } => write!(f, "ori{} {}", get_op_size(*op), AddressingMode::decode_op(*op)),
            ANDI  { op } => todo!(),
            SUBI  { op } => todo!(),
            ADDI  { op } => todo!(),
            EORI  { op } => todo!(),
            CMPI  { op } => todo!(),
            BTST  { op } => todo!(),
            MOVEP { op } => {
                let s = get_op_size_bit9(*op);
                let an = get_op_xn_lo(*op);
                let dn = get_op_xn_hi(*op);
                if get_op_m_hi(*op) & 0b010 != 0 {
                    write!(f, "movep{} d{},(d,a{})", s, dn, an)
                } else {
                    write!(f, "movep{} (d,a{}),d{}", s, an, dn)
                }
            },
            MOVEA { op } => {
                let s = get_op_size_move(*op);

                let am_src = AddressingMode::decode_op(*op);

                let xn = get_op_xn_hi(*op);

                write!(f, "movea{} a{},{}", s, xn, am_src)
            },
            MOVE  { op } => {
                let s = get_op_size_move(*op);

                let am_src = AddressingMode::decode_op(*op);

                let m = get_op_m_hi(*op);
                let xn = get_op_xn_hi(*op);
                let am_dst = AddressingMode::decode(m, xn);

                write!(f, "move{} {},{}", s, am_src, am_dst)
            },
            MOVEToFrom { op } => todo!(),
            NEGX  { op } => todo!(),
            CLR   { op } => todo!(),
            NEG   { op } => todo!(),
            NOT   { op } => write!(f, "not{} {}", get_op_size(*op), AddressingMode::decode_op(*op)),
            EXT   { op } => todo!(),
            NBCD  { op } => todo!(),
            SWAP  { op } => todo!(),
            PEA   { op } => todo!(),
            ILLEGAL            => write!(f, "illegal"),
            TAS   { op } => todo!(),
            TST   { op } => todo!(),
            TRAP  { op } => todo!(),
            LINK  { op } => todo!(),
            UNLK  { op } => todo!(),
            MOVEUSP {op} => todo!(),
            RESET              => write!(f, "reset"),
            NOP                => write!(f, "nop"),
            STOP               => write!(f, "stop"),
            RTE                => write!(f, "rte"),
            RTS                => write!(f, "rts"),
            TRAPV              => write!(f, "trapv"),
            RTR                => write!(f, "rtr"),
            JSR   { op } => todo!(),
            JMP   { op } => todo!(),
            MOVEM { op } => todo!(),
            LEA   { op } => todo!(),
            CHK   { op } => todo!(),
            ADDQ  { op } => todo!(),
            SUBQ  { op } => todo!(),
            Scc   { op } => todo!(),
            DBcc  { op } => todo!(),
            BRA   { op } => todo!(),
            BSR   { op } => todo!(),
            Bcc   { op } => todo!(),
            MOVEQ { op } => write!(f, "moveq #${:02x}", op & 0xff),
            DIVU  { op } => todo!(),
            DIVS  { op } => todo!(),
            SBCD  { op } => todo!(),
            OR    { op } => todo!(),
            SUB   { op } => todo!(),
            SUBX  { op } => todo!(),
            SUBA  { op } => todo!(),
            EOR   { op } => todo!(),
            CMPM  { op } => todo!(),
            CMP   { op } => todo!(),
            CMPA  { op } => todo!(),
            MULU  { op } => todo!(),
            MULS  { op } => todo!(),
            ABCD  { op } => todo!(),
            EXG   { op } => todo!(),
            AND   { op } => todo!(),
            ADD   { op } => todo!(),
            ADDX  { op } => todo!(),
            ADDA  { op } => todo!(),
            ASd   { op } => todo!(),
            LSd   { op } => todo!(),
            ROXd  { op } => todo!(),
            ROd   { op } => todo!(),
            Undefined { op } => write!(f, "undefined ${op:04x}"),
        }
    }
}

impl Instruction {
    pub(crate) fn run(self, cpu: &mut CPU) {
        match self {
            Instruction::ORI   { op } => todo!(),
            Instruction::ANDI  { op } => todo!(),
            Instruction::SUBI  { op } => todo!(),
            Instruction::ADDI  { op } => todo!(),
            Instruction::EORI  { op } => todo!(),
            Instruction::CMPI  { op } => todo!(),
            Instruction::BTST  { op } => todo!(),
            Instruction::MOVEP { op } => movep(op, cpu),
            Instruction::MOVEA { op } => move_(op, cpu),
            Instruction::MOVE  { op } => move_(op, cpu),
            Instruction::MOVEToFrom { op } => todo!(),
            Instruction::NEGX  { op } => todo!(),
            Instruction::CLR   { op } => todo!(),
            Instruction::NEG   { op } => todo!(),
            Instruction::NOT   { op } => not(op, cpu),
            Instruction::EXT   { op } => todo!(),
            Instruction::NBCD  { op } => todo!(),
            Instruction::SWAP  { op } => todo!(),
            Instruction::PEA   { op } => todo!(),
            Instruction::ILLEGAL => todo!(),
            Instruction::TAS   { op } => todo!(),
            Instruction::TST   { op } => todo!(),
            Instruction::TRAP  { op } => todo!(),
            Instruction::LINK  { op } => todo!(),
            Instruction::UNLK  { op } => todo!(),
            Instruction::MOVEUSP { op } => todo!(),
            Instruction::RESET => todo!(),
            Instruction::NOP   => {},   // TODO: "does not begin execution until all pending bus cycles have completed."
            Instruction::STOP  => todo!(),
            Instruction::RTE   => todo!(),
            Instruction::RTS   => todo!(),
            Instruction::TRAPV => todo!(),
            Instruction::RTR   => todo!(),
            Instruction::JSR   { op } => todo!(),
            Instruction::JMP   { op } => todo!(),
            Instruction::MOVEM { op } => todo!(),
            Instruction::LEA   { op } => todo!(),
            Instruction::CHK   { op } => todo!(),
            Instruction::ADDQ  { op } => todo!(),
            Instruction::SUBQ  { op } => todo!(),
            Instruction::Scc   { op } => todo!(),
            Instruction::DBcc  { op } => todo!(),
            Instruction::BRA   { op } => todo!(),
            Instruction::BSR   { op } => todo!(),
            Instruction::Bcc   { op } => todo!(),
            Instruction::MOVEQ { op } => moveq(op, cpu),
            Instruction::DIVU  { op } => todo!(),
            Instruction::DIVS  { op } => todo!(),
            Instruction::SBCD  { op } => todo!(),
            Instruction::OR    { op } => todo!(),
            Instruction::SUB   { op } => todo!(),
            Instruction::SUBX  { op } => todo!(),
            Instruction::SUBA  { op } => todo!(),
            Instruction::EOR   { op } => todo!(),
            Instruction::CMPM  { op } => todo!(),
            Instruction::CMP   { op } => todo!(),
            Instruction::CMPA  { op } => todo!(),
            Instruction::MULU  { op } => todo!(),
            Instruction::MULS  { op } => todo!(),
            Instruction::ABCD  { op } => todo!(),
            Instruction::EXG   { op } => todo!(),
            Instruction::AND   { op } => todo!(),
            Instruction::ADD   { op } => todo!(),
            Instruction::ADDX  { op } => todo!(),
            Instruction::ADDA  { op } => todo!(),
            Instruction::ASd   { op } => todo!(),
            Instruction::LSd   { op } => todo!(),
            Instruction::ROXd  { op } => todo!(),
            Instruction::ROd   { op } => todo!(),
            Instruction::Undefined { op } => panic!("Undefined opcode {op:016b}"),
        }
    }
}


/********************************
 *                              *
 *       OPCODE DECODING        *
 *                              *
 ********************************/

fn get_op_xn_hi(op: u16) -> u8 {
    ((op & 0b111_000_000_000) >> 9) as u8
}

fn get_op_xn_lo(op: u16) -> u8 {
    (op & 0b111) as u8
}

fn get_op_m_hi(op: u16) -> u8 {
    ((op & 0b111_000_000) >> 6) as u8
}

fn get_op_m_lo(op: u16) -> u8 {
    ((op & 0b111_000) >> 3) as u8
}

fn get_op_size(op: u16) -> DataSize {
    // 00 (.b)
    // 01 (.w)
    // 10 (.l)
    (((op & 0b11_000_000) >> 6) as u8).into()
}

fn get_op_size_bit9(op: u16) -> DataSize {
    // 0 -> 01 (.w)
    // 1 -> 10 (.l)
    (((op & 0b1_000_000_000) >> 8) as u8 + 1).into()
}

fn get_op_size_move(op: u16) -> DataSize {
    // 01 -> 00 (.b)
    // 11 -> 01 (.w)
    // 10 -> 10 (.l)
    let s = ((op & 0b11_000_000_000_000) >> 12) as u8;

    let sh = (s & 0b10) >> 1;
    let sl = s & 0b01;

    (((sh & !sl) << 1) | (sh & sl)).into()
}

pub(crate) fn decode_opcode(op: u16) -> Instruction {
    use Instruction::*;

    let op_hi_nibble = (op & 0xf000) >> 12;

    let m = ((op & 0b111000) >> 3) as u8;
    let m_hi = ((op & 0x1c0) >> 6) as u8;

    let d = m_hi & 0b100 != 0;

    let s = m_hi & 0b11;

    let xn = (op & 0b000111) as u8;
    let xn_hi = ((op & 0x0e00) >> 9) as u8;

    match op_hi_nibble {
        0b0000 => {
            let op_hi_byte_bit0_set = (op & 0x0100) != 0;

            if !op_hi_byte_bit0_set {
                let inst_type = xn_hi;

                match inst_type {
                    0b000 => ORI  { op },
                    0b001 => ANDI { op },
                    0b010 => SUBI { op },
                    0b011 => ADDI { op },
                    0b101 => EORI { op },
                    0b110 => CMPI { op },
                    0b100 => BTST { op },
                    _ => Undefined { op },
                }
            } else {
                // b[tst|chg|clr|set], movep
                if m == 0b001 {
                    // movep

                    // Direction:
                    // - true == register to memory
                    // - false == memory to regsiter
                    MOVEP { op }
                } else {
                    // btst, bchg, bclr, bset
                    BTST { op }
                }

            }
        },
        0b0001 | 0b0010 | 0b0011 => {
            // move, movea
            if m_hi == 0b001 {
                // movea
                if op_hi_nibble == 0b00_01 {
                    // movea only has sizes 11 (.w) and 10 (.l)
                    Undefined { op }
                } else {
                    MOVEA { op }
                }
            } else {
                // move
                MOVE { op }
            }
        }
        0b0100 => {
            decode_opcode_0100_prefixed(op, m, m_hi, s, xn, xn_hi)
        },
        0b0101 => {
            if s != 0b11 {
                if m_hi & 0b100 == 0 {
                    ADDQ { op }
                } else {
                    SUBQ { op }
                }
            } else {
                if m != 0b001 {
                    Scc { op }
                } else {
                    DBcc { op }
                }
            }
        },
        0b0110 => {
            let condition = ((op & 0x0f00) >> 8) as u8;

            match condition {
                0 => BRA { op },
                1 => BSR { op },
                _ => Bcc { op },
            }
        },
        0b0111 => {
            if m_hi & 0b100 != 0 {
                MOVEQ { op }
            } else {
                // ?
                Undefined { op }
            }
        },
        0b1000 => {
            if (op & 0x1f0) >> 4 == 0b10000 {
                SBCD { op }
            } else {
                match m_hi {
                    0b011 => DIVU { op },
                    0b111 => DIVS { op },
                    _ => {
                        OR { op }
                    }
                }
            }
        },
        0b1001 => {
            if d && m & 0b110 == 0b000 {
                SUBX { op }
            } else if s == 0b11 {
                SUBA { op }
            } else {
                SUB { op }
            }
        },
        0b1011 => {
            if d && m == 0b001 {
                CMPM { op }
            } else if s == 0b11 {
                CMPA { op }
            } else if !d {
                CMP { op }
            } else {
                EOR { op }
            }
        },
        0b1100 => {
            if (op & 0x1f0) >> 4 == 0b10000 {
                ABCD { op }
            } else if !d && s == 0b11 {
                MULU { op }
            } else if d && s == 0b11 {
                MULS { op }
            } else if d && m & 0b110 == 0b000 {
                EXG { op }
            } else {
                AND { op }
            }
        },
        0b1101 => {
            if d && m & 0b110 == 0b000 {
                ADDX { op }
            } else if s == 0b11 {
                ADDA { op }
            } else {
                ADD { op }
            }
        },
        0b1110 => {
            if s == 0b11 {
                match xn_hi {
                    0b00 => ASd  { op },
                    0b01 => LSd  { op },
                    0b10 => ROXd { op },
                    0b11 => ROd  { op },
                    _ => unreachable!()
                }
            } else {
                match m & 0b11 {
                    0b00 => ASd  { op },
                    0b01 => LSd  { op },
                    0b10 => ROXd { op },
                    0b11 => ROd  { op },
                    _ => unreachable!()
                }
            }
        },
        _ => Undefined { op }
    }
}

#[inline(always)]
fn decode_opcode_0100_prefixed(op: u16, m: u8, m_hi: u8, s: u8, xn: u8, xn_hi: u8) -> Instruction {
    use Instruction::*;

    if m_hi & 0b110 == 0b110 {
        // lea, chk
        if m_hi & 0b001 == 1 {
            return LEA { op }
        } else {
            // NOTE: s is always 0b11 (.w) on mc68000
            // TODO: other chk sizes (mc680x0 where x=20,30,40)
            return CHK { op }
        }
    }

    let op_hi_bit3 = (op & 0x800) >> 8;

    if op_hi_bit3 == 0 {
        let nibble = (op & 0x700) >> 8;

        if s == 0b11 {
            // move [from|to] [sr|ccr]

            match nibble {
                0b000 => MOVEToFrom { op },
                0b100 => MOVEToFrom { op },
                0b110 => MOVEToFrom { op },
                _ => Undefined { op }
            }
        } else {
            // negx, clr, neg, not

            match nibble {
                0b000 => NEGX { op },
                0b010 => CLR  { op },
                0b100 => NEG  { op },
                0b110 => NOT  { op },
                _ => Undefined { op }
            }
        }
    } else {
        match xn_hi {
            0b100 => {
                // ext, nbcd, swap, pea, movem (d == 0)
                if m_hi & 0b110 == 0b010 {
                    if m == 0b000 {
                        EXT { op }
                    } else {
                        MOVEM { op }
                    }
                } else if m_hi == 0b000 {
                    NBCD { op }
                } else if m_hi == 0b001 {
                    if m != 0b000 {
                        SWAP { op }
                    } else {
                        PEA { op }
                    }
                } else {
                    Undefined { op }
                }
            },
            0b101 => {
                // illegal, tas, tst
                if s == 0b11 {
                    if m == 0b111 && xn == 0b100 {
                        ILLEGAL
                    } else {
                        TAS { op }
                    }
                } else {
                    TST { op }
                }
            },
            0b111 => {
                // 0100111xxxxxxxxx
                // trap, link, unlk, move usp, reset, nop, stop, rte, rts, trapv, rtr, jsr
                if m_hi & 0b110 == 0b000 {
                    // 010011100xxxxxxx
                    // ...
                    if m & 0b100 == 0 {
                        // 010011100x0xxxxx
                        // trap, link, unlk
                        match m & 0b11 {
                            0b00 | 0b01 => {
                                TRAP { op }
                            },
                            0b10 => LINK { op },
                            0b11 => UNLK { op },
                            _ => unreachable!(),
                        }
                    } else {
                        // 010011100x1xxxxx
                        if m & 0b010 == 0b000 {
                            // Direction:
                            // - false == register to memory
                            // - true == memory to regsiter
                            MOVEUSP { op }
                        } else {
                            // reset, nop, stop, rte, rts, trapv, rtr
                            // All of the above instructions have only a single opcode.
                            match xn {
                                0b000 => RESET,
                                0b001 => NOP,
                                0b010 => STOP,
                                0b011 => RTE,
                                0b100 => Undefined { op },
                                0b101 => RTS,
                                0b110 => TRAPV,
                                0b111 => RTR,
                                _ => unreachable!(),
                            }
                        }
                    }
                } else {
                    // 0100111XXxxxxxxx
                    // jsr, jmp
                    if m_hi & 0b001 == 0 {
                        JSR { op }
                    } else {
                        JMP { op }
                    }
                }
            },
            0b110 => {
                // movem (d == 1)
                MOVEM { op }
            }
            _ => Undefined { op }
        }
    }
}

/********************************
 *                              *
 * INSTRUCTION IMPLEMENTATIONS  *
 *                              *
 ********************************/

fn move_(op: u16, cpu: &mut CPU) {
    let s = get_op_size_move(op);
    let am_src = AddressingMode::decode_op(op);

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

fn moveq(op: u16, cpu: &mut CPU) {
    let dn = get_op_xn_hi(op).into();
    let val = sign_extend_byte(0xff);
    cpu.reg.write_data(dn, DataSize::Long, val);

    cpu.reg.sr.modify_n(is_negative(val, DataSize::Long));
    cpu.reg.sr.modify_z(is_zero(val, DataSize::Long));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}

fn movep(op: u16, cpu: &mut CPU) {
    let displacement = cpu.read_pc_inc_short();

    let an: RegisterNum = get_op_xn_lo(op).into();
    let dn: RegisterNum = get_op_xn_hi(op).into();

    let m_hi = get_op_m_hi(op);
    let direction = m_hi & 0b010 != 0;
    let size = get_op_size_bit9(op);

    // TODO: check if this is correct
    if direction {
        // reg -> mem
        println!("MOVEP REG -> MEM");
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
        println!("MOVEP MEM -> REG");
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

fn not(op: u16, cpu: &mut CPU) {
    let s = get_op_size(op);
    let am = AddressingMode::decode_op(op);

    let res = !cpu.read_am(s, am);
    cpu.cycle_when_long(s, 1);
    cpu.write_am(s, am, res);
    cpu.cycle_when_long(s, 1);

    cpu.reg.sr.modify_n(is_negative(res, s));
    cpu.reg.sr.modify_z(is_zero(res, s));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}


