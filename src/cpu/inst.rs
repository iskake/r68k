enum AddressingMode {
    DataRegDir,
    AddrRegDir,
    AddrRegInd,
    AddrRegIndPostInc,
    AddrRegIndPreDec,
    AddrRegIndDisplace,
    AddrRegIndIdx8,
    AddrRegIndIdxBase,
    MemIndPostIdx,
    MemIndPreIdx,
    PCIndDisplace,
    PCIndIdx8,
    PCMemIndPostIdx,
    PCMemIndPreIdx,
    Immediate,
}

fn get_addr_mode(opcode: u32) -> AddressingMode {
    use AddressingMode::*;

    let reg = opcode & 0b111;
    let mode = (opcode & 0b111000) >> 3;

    match mode {
        0b000 => DataRegDir,
        0b001 => AddrRegDir,
        0b010 => AddrRegIndPostInc,
        0b011 => AddrRegIndPreDec,
        0b100 => AddrRegIndDisplace,
        0b101 => AddrRegIndIdx8,
        0b110 => {
            // Either postindexed or preindexed

            // AddrRegIndIdx8 requires 1 extension word
            // AddrRegIndIdxBase requires 1,2, or 3 extension word(s)
            // MemIndPostIdx requires 1,2,3,4, or 5 extension word(s)
            todo!()
        },
        0b111 => {
            match reg {
                0b010 => PCIndDisplace,
                0b011 => {
                    // Either of 8bit displacement or base displacement
                    todo!()
                }
                0b000 => {
                    // Either of postindexed or preindexed
                    todo!()
                }
                0b100 => Immediate,
                _ => todo!()
            }
        }
        _ => unreachable!(),
    }
}

#[derive(Debug)]
pub enum Instruction {
    // 0000
    ORItoCCR, ORItoSR, ORI, ANDItoCCR, ANDItoSR, ANDI, SUBI, ADDI,
    EORItoCCR, EORItoSR, EORI, CMPI, BTST, BCHG, BCLR, BSET, MOVEP, 
    // BTST, BCHG, BCLR, BSET, MOVEP (Dn)
    // 00ss
    MOVEA, MOVE,
    // 0100
    MOVEfromSR, MOVEtoCCR, MOVEtoSR, NEGX, CLR, NEG, NOT, EXT, NBCD, SWAP,
    PEA, ILLEGAL, TAS, TST, TRAP, LINK, UNLK, MOVEUSP, RESET, NOP, STOP,
    RTE, RTS, TRAPV, RTR, JSR, JMP, MOVEM, LEA, CHK,
    // 0101
    ADDQ, SUBQ, Scc, DBcc,
    // 0110
    BRA, BSR, Bcc,
    // 0111
    MOVEQ,
    // 1000
    DIVU, DIVS, SBCD, OR,
    // 1001
    SUB, SUBX, SUBA,
    // 1010
    // ...
    // 1011
    EOR, CMPM, CMP, CMPA,
    // 1100
    MULU, MULS, ABCD, EXG, AND,
    // 1101
    ADD, ADDX, ADDA,
    // 1110
    ASd, LSd, ROXd, ROd,
    // ASd, LSd, ROXd, ROd (rotation)
    // 1111
    // Floating point/supervisor instructions?

    // Extra
    Undefined(u16),
}

pub(crate) fn get_instr(op: u16) -> Instruction {
    use Instruction::*;

    let hi = (op & 0xf000) >> 12;

    let s = (op & 0xc0) >> 6;
    let m = (op & 0b111000) >> 3;
    let xan = op & 0b000111;

    match hi {
        0b0000 => {
            let bx_data = (op & 0x0100) != 0x0100;

            let a = (op & 0x0e00) >> 9;

            if bx_data {
                // b[tst|chg|clr|set], movep
                let data_reg = a;

                if m == 0b001 {
                    // movep
                    let direction = (s & 0b10) != 0;
                    let size = (s & 0b01) != 0;
                    let addr_reg = xan;

                    MOVEP
                } else {
                    match s {
                        0b00 => BTST,
                        0b01 => BCHG,
                        0b10 => BCLR,
                        0b11 => BSET,
                        _ => unreachable!()
                    }
                }
            } else {
                let inst_type = a;

                match inst_type {
                    0b000 => ORI,
                    0b001 => ANDI,
                    0b010 => SUBI,
                    0b011 => ADDI,
                    0b100 => BTST,
                    0b101 => EORI,
                    0b110 => CMPI,
                    _ => Undefined(op),
                }
            }
        },
        0b0001 | 0b0010 | 0b0011 => {
            // move, movea
            let s = (op & 0x3000) >> 12;
            let xan_ = (op & 0xe00) >> 9;
            let m_ = (op & 0x1c0) >> 6;

            if m_ == 0b001 {
                // movea
                MOVEA
            } else {
                // move
                MOVE
            }
        }
        0b0100 => {
            if (op & 0x800) == 0 {
                if s == 0b11 {
                    // move [from|to] [sr|ccr]
                    let nibble = (op & 0x0700) >> 8;

                    match nibble {
                        0b000 => MOVEfromSR,
                        0b100 => MOVEtoCCR,
                        0b110 => MOVEtoSR,
                        _ => Undefined(op)
                    }
                } else {
                    // negx, clr, neg, not
                    Undefined(op)
                }
            } else {
                // ...
                Undefined(op)
            }
        },
        0b0101 => {
            Undefined(op)
        },
        0b0110 => {
            Undefined(op)
        },
        0b0111 => {
            Undefined(op)
        },
        0b1000 => {
            Undefined(op)
        },
        0b1001 => {
            Undefined(op)
        },
        0b1011 => {
            Undefined(op)
        },
        0b1100 => {
            Undefined(op)
        },
        0b1101 => {
            Undefined(op)
        },
        0b1110 => {
            Undefined(op)
        },
        _ => Undefined(op)
    }
}

fn _x() {
    let _insts = [
        // NOTATION:
        // f -- flag
        // r -- (some) register 
        // x -- (some) register x
        // y -- (some) register y
        // d -- data register (d0-d7)
        // a -- address register (a0-a7)
        // o -- opmode
        // e -- effective address
        // s -- size (00: byte, 01: word, 10: long, 11: ?)
        // c -- condition
        // D -- (literal immediate) data
        // P -- disPlacement
        // C -- "count?register", either shift count or register
        ("abcd",  "1100xxx10000fyyy"),
        ("add",   "1101rrroooeeeeee"),
        ("adda",  "1101rrroooeeeeee"),
        ("addi",  "00000110sseeeeee"),
        ("addq",  "0101DDD0sseeeeee"),
        ("addx",  "1101xxx1ss00fyyy"),
        ("and",   "1100rrroooeeeeee"),
        ("andi",  "00000010sseeeeee"),
        ("andi ccr",  "000000100011110000000000DDDDDDDD"),
        ("asX",   "1110CCCfssf00rrr"),
        ("asX",   "1110000f11eeeeee"),
        ("bcc",   "0110ccccPPPPPPPP"),
        ("bchg",  "0000rrr101eeeeee"),
        ("or",    "1000rrroooeeeeee"),
        ("move",  "00ssrrrmmmmmmrrr"),
        ("movea", "00ssrrr001mmmrrr"),
        ("movem", "01001d001smmmrrr"),
        ("movep", "0000dddooo001aaa"),
        ("moveq", "0111rrr0dddddddd"),
    ];
}