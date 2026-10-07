enum AddressingMode {
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
    fn get(mode: u8, reg: u8) -> AddressingMode {
        use AddressingMode::*;

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
}

#[derive(Debug)]
enum VariationCCRSR {
    Normal,
    ToCCR,
    ToSR,
}

#[derive(Debug)]
pub enum Instruction {
    // 0000
    ORI  { s: u8, m: u8, xn: u8, variation: VariationCCRSR },
    ANDI { s: u8, m: u8, xn: u8, variation: VariationCCRSR },
    SUBI { s: u8, m: u8, xn: u8 },
    ADDI { s: u8, m: u8, xn: u8 },
    EORI { s: u8, m: u8, xn: u8, variation: VariationCCRSR },
    CMPI { s: u8, m: u8, xn: u8 },
    BTST { dn: u8, b_variation: u8, m: u8, xn: u8 },
    MOVEP { dn: u8, d: bool, s: bool, an: u8 },
    // BTST, BCHG, BCLR, BSET, MOVEP (Dn)
    // 00ss
    MOVEA { s: u8, an: u8, m: u8, xn: u8 },
    MOVE  { s: u8, xn_hi: u8, m_hi: u8, m_lo: u8, xn_lo: u8 },
    // 0100
    MOVEToFrom { m: u8, xn: u8, variation: VariationCCRSR },
    NEGX { s: u8, m: u8, xn: u8 },
    CLR  { s: u8, m: u8, xn: u8 },
    NEG  { s: u8, m: u8, xn: u8 },
    NOT  { s: u8, m: u8, xn: u8 },
    EXT  { s: bool, dn: u8 },
    NBCD { m: u8, xn: u8 },
    SWAP { dn: u8 },
    PEA  { m: u8, xn: u8 },
    ILLEGAL,
    TAS  { m: u8, xn: u8 },
    TST  { s: u8, m: u8, xn: u8},
    TRAP { vector: u8 },
    LINK { an: u8 },
    UNLK { an: u8 },
    MOVEUSP { d: bool, an: u8 },
    RESET, NOP, STOP,
    RTE, RTS, TRAPV, RTR,
    JSR { m: u8, xn: u8 },
    JMP { m: u8, xn: u8 },
    MOVEM { d: bool, s: bool, m: u8, xn: u8 },
    LEA { an: u8, m: u8, xn: u8 },
    CHK { dn: u8, s: u8, m: u8, xn: u8 },
    // 0101
    ADDQ { data: u8, s: u8, m: u8, xn: u8 },
    SUBQ { data: u8, s: u8, m: u8, xn: u8 },
    Scc  { condition: u8, m: u8, xn: u8 },
    DBcc { condition: u8, dn: u8 },
    // 0110
    BRA { displacement: u8 },
    BSR { displacement: u8 },
    Bcc { condition: u8, displacement: u8 },
    // 0111
    MOVEQ { dn: u8, data: u8 },
    // 1000
    DIVU { dn: u8, m: u8, xn: u8 },
    DIVS { dn: u8, m: u8, xn: u8 },
    SBCD { xn_hi: u8, m: bool, xn_lo: u8 },
    OR   { dn: u8, d: bool, s: u8, m: u8, xn: u8 },
    // 1001
    SUB  { dn: u8, d: bool, s: u8, m: u8, xn: u8 },
    SUBX { xn_hi: u8, s: u8, m: bool, xn_lo: u8 },
    SUBA { an: u8, s: bool, m: u8, xn: u8 },
    // 1010
    // ...
    // 1011
    EOR  { dn: u8, s: u8, m: u8, xn: u8 },
    CMPM { an_hi: u8, s: u8, an_lo: u8 },
    CMP  { dn: u8, s: u8, m: u8, xn: u8 },
    CMPA { an: u8, s: bool, m: u8, xn: u8 },
    // 1100
    MULU { dn: u8, m: u8, xn: u8 },
    MULS { dn: u8, m: u8, xn: u8 },
    ABCD { xn_hi: u8, m: bool, xn_lo: u8 },
    EXG  { xn_hi: u8, m: u8, xn: u8 },
    AND  { dn: u8, d: bool, s: u8, m: u8, xn: u8 },
    // 1101
    ADD  { dn: u8, d: bool, s: u8, m: u8, xn: u8 },
    ADDX { xn_hi: u8, s: u8, m: bool, xn_lo: u8 },
    ADDA { an: u8, s: bool, m: u8, xn: u8 },
    // 1110
    ASd  { rot: u8, d: bool, s: u8, m: u8, xn: u8 },
    LSd  { rot: u8, d: bool, s: u8, m: u8, xn: u8 },
    ROXd { rot: u8, d: bool, s: u8, m: u8, xn: u8 },
    ROd  { rot: u8, d: bool, s: u8, m: u8, xn: u8 },
    // ASd, LSd, ROXd, ROd (rotation)
    // 1111
    // Floating point/supervisor instructions?

    // Extra
    Undefined(u16),
}

pub(crate) fn decode(op: u16) -> Instruction {
    use Instruction::*;

    let op_hi_nibble = (op & 0xf000) >> 12;
    let op_lo_byte = (op & 0xff) as u8;

    let m = ((op & 0b111000) >> 3) as u8;
    let m_hi = ((op & 0x1c0) >> 6) as u8;

    let s = m_hi & 0b11;

    let xn = (op & 0b000111) as u8;
    let xn_hi = ((op & 0x0e00) >> 9) as u8;

    match op_hi_nibble {
        0b0000 => {
            let op_hi_bit0 = (op & 0x0100) != 0x0100;

            if op_hi_bit0 {
                // b[tst|chg|clr|set], movep
                let data_reg = xn_hi;

                if m == 0b001 {
                    // movep

                    // Direction:
                    // - true == register to memory
                    // - false == memory to regsiter
                    let direction = (s & 0b10) != 0;
                    let size = (s & 0b01) != 0;

                    MOVEP { dn: data_reg, d: direction, s: size, an: xn }
                } else {
                    // btst, b
                    BTST { dn: xn_hi, b_variation: s, m, xn }
                }
            } else {
                let inst_type = xn_hi;

                match inst_type {
                    0b000 => ORI  { s, m, xn, variation: VariationCCRSR::Normal},
                    0b001 => ANDI { s, m, xn, variation: VariationCCRSR::Normal},
                    0b010 => SUBI { s, m, xn},
                    0b011 => ADDI { s, m, xn},
                    0b100 => BTST { dn: 0b100, b_variation: s, m, xn},
                    0b101 => EORI { s, m, xn, variation: VariationCCRSR::Normal},
                    0b110 => CMPI { s, m, xn},
                    _ => Undefined(op),
                }
            }
        },
        0b0001 | 0b0010 | 0b0011 => {
            // move, movea
            let s = ((op & 0x3000) >> 12) as u8;

            if m_hi == 0b001 {
                // movea
                MOVEA { s, an: xn_hi, m, xn }
            } else {
                // move
                MOVE { s, xn_hi, m_hi, m_lo: m, xn_lo: xn }
            }
        }
        0b0100 => {
            decode_0100_prefixed(op, m, m_hi, s, xn, xn_hi)
        },
        0b0101 => {
            if s != 0b11 {
                if m_hi & 0b100 == 0 {
                    ADDQ { data: xn_hi, s, m, xn }
                } else {
                    SUBQ { data: xn_hi, s, m, xn }
                }
            } else {
                let condition = ((op & 0x0f00) >> 8) as u8;

                if m != 0b001 {
                    Scc { condition, m, xn }
                } else {
                    DBcc { condition, dn: xn }
                }
            }
        },
        0b0110 => {
            let condition = ((op & 0x0f00) >> 8) as u8;

            match condition {
                0 => BRA { displacement: op_lo_byte },
                1 => BSR { displacement: op_lo_byte },
                _ => Bcc { condition, displacement: op_lo_byte },
            }
        },
        0b0111 => {
            if m_hi & 0b100 != 0 {
                MOVEQ { dn: xn_hi, data: op_lo_byte }
            } else {
                // ?
                Undefined(op)
            }
        },
        0b1000 => {
            if (op & 0x1f0) >> 4 == 0b10000 {
                let m = m & 0b001 == 1;
                SBCD { xn_hi, m, xn_lo: xn }
            } else {
                match m_hi {
                    0b011 => DIVU { dn: xn_hi, m, xn },
                    0b111 => DIVS { dn: xn_hi, m, xn },
                    _ => {
                        let direction = m_hi & 0b100 != 0;
                        OR { dn: xn_hi, d: direction, s, m, xn }
                    }
                }
            }
        },
        0b1001 => {
            let d = m_hi & 0b100 != 0;

            if d && m & 0b110 == 0b000 {
                let m = m & 0b001 == 1;
                SUBX { xn_hi, s, m, xn_lo: xn}
            } else if s == 0b11 {
                let s = m_hi & 0b100 != 0;
                SUBA { an: xn_hi, s, m, xn }
            } else {
                SUB { dn: xn_hi, d, s, m, xn }
            }
        },
        0b1011 => {
            let d = m_hi & 0b100 != 0;

            if d && m == 0b001 {
                CMPM { an_hi: xn_hi, s, an_lo: xn }
            } else if s == 0b11 {
                let s = m_hi & 0b100 != 0;
                CMPA { an: xn_hi, s, m, xn }
            } else if !d {
                CMP { dn: xn_hi, s, m, xn }
            } else {
                EOR { dn: xn_hi, s, m, xn }
            }
        },
        0b1100 => {
            let d = m_hi & 0b100 != 0;

            if (op & 0x1f0) >> 4 == 0b10000 {
                let m = m & 0b001 == 1;

                ABCD { xn_hi, m, xn_lo: xn }
            } else if !d && s == 0b11 {
                MULU { dn: xn_hi, m, xn }
            } else if d && s == 0b11 {
                MULS { dn: xn_hi, m, xn }
            } else if d && m & 0b110 == 0b000 {
                let m = ((op_lo_byte & 0b1111_1000) >> 3) as u8;

                EXG { xn_hi, m, xn }
            } else {
                AND { dn: xn_hi, d, s, m, xn }
            }
        },
        0b1101 => {
            let d = m_hi & 0b100 != 0;

            if d && m & 0b110 == 0b000 {
                let m = m & 0b001 == 1;
                ADDX { xn_hi, s, m, xn_lo: xn}
            } else if s == 0b11 {
                let s = m_hi & 0b100 != 0;
                ADDA { an: xn_hi, s, m, xn }
            } else {
                ADD { dn: xn_hi, d, s, m, xn }
            }
        },
        0b1110 => {
            let d = m_hi & 0b100 != 0;

            //? Should the handling of `s == 0b11` and `s != 0b11` be different?
            match xn_hi {
                0b00 => ASd  { rot: xn_hi, d, s, m, xn },
                0b01 => LSd  { rot: xn_hi, d, s, m, xn },
                0b10 => ROXd { rot: xn_hi, d, s, m, xn },
                0b11 => ROd  { rot: xn_hi, d, s, m, xn },
                _ => unreachable!()
            }
        },
        _ => Undefined(op)
    }
}

#[inline(always)]
fn decode_0100_prefixed(op: u16, m: u8, m_hi: u8, s: u8, xn: u8, xn_hi: u8) -> Instruction {
    use Instruction::*;

    if m_hi & 0b110 == 0b110 {
        // lea, chk
        if m_hi & 0b001 == 1 {
            return LEA { an: xn_hi, m, xn }
        } else {
            // NOTE: s is always 0b11 (.w) on mc68000
            // TODO: other chk sizes (mc680x0 where x=20,30,40)
            let s = (m_hi & 0b110) >> 1;
            return CHK { dn: xn_hi, s, m, xn }
        }
    }

    let op_hi_bit3 = (op & 0x800) >> 8;

    if op_hi_bit3 == 0 {
        let nibble = (op & 0x700) >> 8;

        if s == 0b11 {
            // move [from|to] [sr|ccr]

            match nibble {
                0b000 => MOVEToFrom { m, xn, variation: VariationCCRSR::Normal },
                0b100 => MOVEToFrom { m, xn, variation: VariationCCRSR::ToCCR },
                0b110 => MOVEToFrom { m, xn, variation: VariationCCRSR::ToSR },
                _ => Undefined(op)
            }
        } else {
            // negx, clr, neg, not

            match nibble {
                0b000 => NEGX { s, m, xn },
                0b010 => CLR  { s, m, xn },
                0b100 => NEG  { s, m, xn },
                0b110 => NOT  { s, m, xn },
                _ => Undefined(op)
            }
        }
    } else {
        match xn_hi {
            0b100 => {
                // ext, nbcd, swap, pea, movem (d == 0)
                if m_hi & 0b110 == 0b010 {
                    let s = m_hi & 0b001 == 1;

                    if m == 0b000 {
                        EXT { s, dn: xn }
                    } else {
                        MOVEM { d: false, s, m, xn }
                    }
                } else if m_hi == 0b000 {
                    NBCD { m, xn }
                } else if m_hi == 0b001 {
                    if m != 0b000 {
                        SWAP { dn: xn }
                    } else {
                        PEA { m, xn }
                    }
                } else {
                    Undefined(op)
                }
            },
            0b101 => {
                // illegal, tas, tst
                if s == 0b11 {
                    if m == 0b111 && xn == 0b100 {
                        ILLEGAL
                    } else {
                        TAS { m, xn }
                    }
                } else {
                    TST { s, m, xn }
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
                                let vector = (op & 0xf) as u8;
                                TRAP { vector }
                            },
                            0b10 => LINK { an: xn },
                            0b11 => UNLK { an: xn },
                            _ => unreachable!(),
                        }
                    } else {
                        // 010011100x1xxxxx
                        if m & 0b010 == 0b000 {
                            // Direction:
                            // - false == register to memory
                            // - true == memory to regsiter
                            let direction = m & 0b001 == 1;
                            MOVEUSP { d: direction, an: xn }
                        } else {
                            // reset, nop, stop, rte, rts, trapv, rtr
                            // All of the above instructions have only a single opcode.
                            match xn {
                                0b000 => RESET,
                                0b001 => NOP,
                                0b010 => STOP,
                                0b011 => RTE,
                                0b100 => Undefined(op),
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
                        JSR { m, xn }
                    } else {
                        JMP { m, xn }
                    }
                }
            },
            0b110 => {
                // movem (d == 1)
                let s = m_hi & 0b001 == 1;
                MOVEM { d: true, s, m, xn }
            }
            _ => Undefined(op)
        }
    }
}
