use crate::{bits::DataSize, cpu::inst::Instruction};

pub(crate) fn get_op_xn_hi(op: u16) -> u8 {
    ((op & 0b111_000_000_000) >> 9) as u8
}

pub(crate) fn get_op_xn_lo(op: u16) -> u8 {
    (op & 0b111) as u8
}

pub(crate) fn get_op_m_hi(op: u16) -> u8 {
    ((op & 0b111_000_000) >> 6) as u8
}

pub(crate) fn get_op_m_lo(op: u16) -> u8 {
    ((op & 0b111_000) >> 3) as u8
}

pub(crate) fn get_op_size(op: u16) -> DataSize {
    // 00 (.b)
    // 01 (.w)
    // 10 (.l)
    (((op & 0b11_000_000) >> 6) as u8).into()
}

pub(crate) fn get_op_size_bit6(op: u16) -> DataSize {
    // 0 -> 01 (.w)
    // 1 -> 10 (.l)
    (((op & 0b1_000_000) >> 6) as u8 + 1).into()
}

pub(crate) fn get_op_size_bit8(op: u16) -> DataSize {
    // 0 -> 01 (.w)
    // 1 -> 10 (.l)
    (((op & 0b100_000_000) >> 8) as u8 + 1).into()
}

pub(crate) const SIZE_SHIFT_CHK:  u8 = 7;
pub(crate) const SIZE_SHIFT_MOVE: u8 = 12;

pub(crate) fn get_op_size_alt(op: u16, shift: u8) -> DataSize {
    // 01 -> 00 (.b)
    // 11 -> 01 (.w)
    // 10 -> 10 (.l)
    let s = ((op & (0b11 << shift)) >> shift) as u8;

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

                // All instructions other than bset have s != 11
                if s == 0b11 {
                    if inst_type == 0b100 {
                        // bset
                        BSET { op } 
                    } else {
                        Undefined { op }
                    }
                } else {
                    match inst_type {
                        0b000 => if m == 0b111 && xn == 0b100 {
                            if s == 0b00 {
                                ORIToCCR
                            } else if s == 0b01 {
                                ORIToSR
                            } else {
                                Undefined { op }
                            }
                        } else {
                            ORI  { op }
                        },
                        0b001 => if m == 0b111 && xn == 0b100 {
                            if s == 0b00 {
                                ANDIToCCR
                            } else if s == 0b01 {
                                ANDIToSR
                            } else {
                                Undefined { op }
                            }
                        } else {
                            ANDI { op }
                        },
                        0b101 => if m == 0b111 && xn == 0b100 {
                            if s == 0b00 {
                                EORIToCCR
                            } else if s == 0b01 {
                                EORIToSR
                            } else {
                                Undefined { op }
                            }
                        } else {
                            EORI { op }
                        },
                        0b010 => SUBI { op },
                        0b011 => ADDI { op },
                        0b110 => CMPI { op },
                        0b100 => {
                            match s {
                                0b00 => BTST { op },
                                0b01 => BCHG { op },
                                0b10 => BCLR { op },
                                0b11 => BSET { op },
                                _ => unreachable!(),
                            }
                        },
                        _ => Undefined { op },
                    }
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
                    match s {
                        0b00 => BTST { op },
                        0b01 => BCHG { op },
                        0b10 => BCLR { op },
                        0b11 => BSET { op },
                        _ => unreachable!(),
                    }
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
            if m_hi & 0b100 == 0 {
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
            if s == 0b11 {
                SUBA { op }
            } else if d && m & 0b110 == 0b000 {
                SUBX { op }
            } else {
                SUB { op }
            }
        },
        0b1011 => {
            if s == 0b11 {
                CMPA { op }
            } else if d && m == 0b001 {
                CMPM { op }
            } else if !d {
                CMP { op }
            } else {
                if m_hi & 0b100 != 0 {
                    // eor only has <ea> ^ dn -> <ea>
                    EOR { op }
                } else {
                    Undefined { op }
                }

            }
        },
        0b1100 => {
            if (op & 0x1f0) >> 4 == 0b10000 {
                ABCD { op }
            } else if !d && s == 0b11 {
                MULU { op }
            } else if d && s == 0b11 {
                MULS { op }
            } else if d
                   && m & 0b110 == 0b000
                   && 0b00 < s
                   && s < 0b11 {
                if !((s == 0b10) && (m & 1 == 0)) {
                    EXG { op }
                } else {
                    Undefined { op }
                }
            } else {
                AND { op }
            }
        },
        0b1101 => {
            if s == 0b11 {
                ADDA { op }
            } else if d && m & 0b110 == 0b000 {
                ADDX { op }
            } else {
                ADD { op }
            }
        },
        0b1110 => {
            if s == 0b11 {
                match xn_hi {
                    0b00 => ASdM  { op },
                    0b01 => LSdM  { op },
                    0b10 => ROXdM { op },
                    0b11 => ROdM  { op },
                    _ => unreachable!()
                }
            } else {
                match m & 0b11 {
                    0b00 => ASdD  { op },
                    0b01 => LSdD  { op },
                    0b10 => ROXdD { op },
                    0b11 => ROdD  { op },
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
            // s is always 0b11 (.w) on mc68000
            return CHK { op }
        }
    } else if m_hi & 0b110 == 0b100 {
        // TODO: revision checking in decode fn

        // MC680x0, for x=2|3|4 only
        // CHK { op }
        return Undefined { op }
    }

    let op_hi_bit3 = (op & 0x800) >> 8;

    if op_hi_bit3 == 0 {
        let nibble = (op & 0x700) >> 8;

        if s == 0b11 {
            // move [from|to] [sr|ccr]

            match nibble {
                0b000 => MOVEFromSR { op },
                0b100 => MOVEToCCR  { op },
                0b110 => MOVEToSR   { op },
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
                    if m == 0b000 {
                        SWAP { op }
                    } else {
                        PEA { op }
                    }
                } else if m_hi == 0b111 {
                    // TODO: EXTB
                    // EXTB { op }
                    Undefined { op }
                } else {
                    Undefined { op }
                }
            },
            0b101 => {
                // illegal, tas, tst
                if s == 0b11 {
                    if m == 0b111 && xn == 0b100 {
                        ILLEGAL { op }
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
