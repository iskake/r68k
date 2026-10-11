use crate::{bits::{is_negative, is_zero}, cpu::CPU};

enum DataSize {
    Byte,
    Short,
    Long,
    Any,
}

enum DataType {
    Immediate,
    BitIndex,
    Displacement,
    DisplacementOptional,
    RegisterListMask,
}

pub(crate) enum AddressingMode {
    /// Data register `Dn`
    DataReg(u8),
    /// Address register `An`
    AddrReg(u8),
    /// Indirect address `(An)`
    Addr(u8),
    /// Address with postincrement `(An)+`
    AddrPostInc(u8),
    /// Address with predecrement `-(An)`
    AddrPreDec(u8),
    /// Address with displacement `(d16, An)`
    AddrDisplace(u8),
    /// Address with index `(d8, An, Xn)`
    /// - AddrRegIndIdx8 requires 1 extension word
    /// - AddrRegIndIdxBase requires 1,2, or 3 extension word(s)
    /// - MemIndPostIdx requires 1,2,3,4, or 5 extension word(s)
    /// - MemIndPreIdx requires 1,2,3,4, or 5 extension word(s)
    AddrIdx(u8),
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

impl AddressingMode {
    fn decode(mode: u8, reg: u8) -> AddressingMode {
        use AddressingMode::*;

        assert_eq!(mode & 0b111, reg);
        assert_eq!(reg  & 0b111, mode);

        match mode {
            0b000 => DataReg(reg),
            0b001 => AddrReg(reg),
            0b010 => Addr(reg),
            0b011 => AddrPostInc(reg),
            0b100 => AddrPreDec(reg),
            0b101 => AddrDisplace(reg),
            0b110 => AddrIdx(reg),
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
        let mode = (op & 0b111_000 >> 3) as u8;
        let reg  = (op & 0b000_111) as u8;
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
            Instruction::MOVEP { op } => todo!(),
            Instruction::MOVEA { op } => todo!(),
            Instruction::MOVE  { op } => todo!(),
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
            Instruction::NOP   => {},
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
            Instruction::MOVEQ { op } => todo!(),
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
            Instruction::Undefined { op } => panic!("Undefined opcode {op:016x}"),
        }
    }
}

fn not(op: u16, cpu: &mut CPU) {
    let s = get_op_size(op);
    let am = AddressingMode::decode_op(op);

    let res = !cpu.read_am(s, am);

    cpu.reg.sr.modify_n(is_negative(res, s));
    cpu.reg.sr.modify_z(is_zero(res, s));
    cpu.reg.sr.clear_v();
    cpu.reg.sr.clear_c();
}

fn get_op_size(op: u16) -> u8 {
    ((op & 0b11_000_000) >> 6) as u8
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
                MOVEA { op }
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
