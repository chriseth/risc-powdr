use std::{fs, path::Path};

use crate::asm_parser::{self, Argument, Register, Statement};

pub fn compile_file(file: &Path) {
    compile_riscv_asm(&fs::read_to_string(file).unwrap());
}

pub fn compile_riscv_asm(data: &str) {
    let data = data.to_string() + &library_routines();
    print!("{}", preamble());

    for s in asm_parser::parse_asm(&data) {
        print!("{}", process_statement(s));
    }
}

fn preamble() -> String {
    r#"
reg pc[@pc];
reg X[<=];
reg Y[<=];
reg Z[<=];
"#
    .to_string()
        + &(0..32)
            .map(|i| format!("reg x{i};\n"))
            .collect::<Vec<_>>()
            .concat()
        + r#"
reg ADDR;

pil{
    x0 = 0;
}

pil{
// ============== iszero check for X =======================
    col witness XInv;
    col witness XIsZero;
    XIsZero = 1 - X * XInv;
    XIsZero * X = 0;
    XIsZero * (1 - XIsZero) = 0;

// =============== read-write memory =======================
    // Read-write memory. Columns are sorted by m_addr and
    // then by m_step. m_change is 1 if and only if m_addr changes
    // in the next row.
    col witness m_addr;
    col witness m_step;
    col witness m_change;
    col witness m_value;
    // If we have an operation at all (needed because this needs to be a permutation)
    col witness m_op;
    // If the operation is a write operation.
    col witness m_is_write;
    col witness m_is_read;

    // positive numbers (assumed to be much smaller than the field order)
    col fixed POSITIVE(i) { i + 1 };
    col fixed FIRST = [1];
    col fixed LAST(i) { FIRST(i + 1) };
    col fixed STEP(i) { i };

    m_change * (1 - m_change) = 0;

    // if m_change is zero, m_addr has to stay the same.
    (m_addr' - m_addr) * (1 - m_change) = 0;

    // Except for the last row, if m_change is 1, then m_addr has to increase,
    // if it is zero, m_step has to increase.
    (1 - LAST) { m_change * (m_addr' - m_addr) + (1 - m_change) * (m_step' - m_step) } in POSITIVE;

    m_op * (1 - m_op) = 0;
    m_is_write * (1 - m_is_write) = 0;
    m_is_read * (1 - m_is_read) = 0;
    // m_is_write can only be 1 if m_op is 1.
    m_is_write * (1 - m_op) = 0;
    m_is_read * (1 - m_op) = 0;
    m_is_read * m_is_write = 0;


    // If the next line is a read and we stay at the same address, then the
    // value cannot change.
    (1 - m_is_write') * (1 - m_change) * (m_value' - m_value) = 0;

    // If the next line is a read and we have an address change,
    // then the value is zero.
    (1 - m_is_write') * m_change * m_value' = 0;
}

// ============== memory instructions ==============

instr mstore <=X= val { { ADDR, STEP, X } is m_is_write { m_addr, m_step, m_value } }
instr mload r <=X= { { ADDR, STEP, X } is m_is_read { m_addr, m_step, m_value } }

// ============== control-flow instructions ==============

instr jump l: label { pc' = l }
instr call l: label { pc' = l, x1' = pc + 1, x6' = l }
instr ret { pc' = x1 }

instr branch_if_nonzero <=X= c, l: label { pc' = (1 - XIsZero) * l + XIsZero * (pc + 1) }}
instr branch_if_zero <=X= c, l: label { pc' = XIsZero * l + (1 - XIsZero) * (pc + 1) }}

// input X is required to be the difference of two 32-bit unsigend values.
// i.e. -2**32 < X < 2**32
instr branch_if_positive <=X= c, l: label {
    X = Xhi * 2**16 + Xlo - wrap_bit * 2**32 + 1,
    pc' = wrap_bit * l + (1 - wrap_bit) * (pc + 1)
}

// ================= logical instructions =================

instr is_equal_zero <=X= v, t <= Y { Y = XIsZero }

// ================= arith/bitwise instructions =================

instr xor <=X= a, <=Y= b, c <= Z {
    {X, Y, Z} in 1 { binary.X, binary.Y, binary.RESULT, 1 }
}
// we wanted better synatx: { binary(X, Y, Z) }
// maybe alternate syntax: instr xor a(Y), b(Z) -> X


// ================== wrapping instructions ==============

// Wraps a value in Y to 32 bits.
// Requires 0 <= Y < 2**33
instr wrap <=Y= v, x <= X { Y = X + wrap_bit * 2**32, X = Xhi * 2**16 + Xlo }
pil{
    Xlo in bytes2;
    Xhi in bytes2;
    col commit wrap_bit;
    wrap_bit * (1 - wrap_bit) = 0;
}

// ======================= assertions =========================

instr fail { 1 = 0; }

// Removes up to 16 bits beyond 32
// TODO is this really safe?
instr wrap16 <=X= v, t <=Y= { X = Xupper * 2*32 + Xhi * 2**16 + Xlo, Y = Xhi * 2**16 + Xlo }
pil{
    col commit Xupper;
    Xupper in bytes2;
}

// set the stack pointer.
// TODO other things to initialize?
x2 <=X= 0x10000
    "#
}

fn library_routines() -> String {
    r#"
memset@plt:
# a4: number of bytes
# a0: memory location
# a1: value
# We assume the value is zero and a4 is a multiple of 4
   beqz a4, ___end_memset
   sw a1, 0(a0)
   addi a4, a4, -4
   j memset@plt
___end_memset:
  ret

_ZN4core9panicking18panic_bounds_check17hdf372c1f1d454407E:
_ZN4core9panicking5panic17h3bc01ca1a5023c7aE:
_ZN4core5slice5index24slice_end_index_len_fail17hdcde4291d30716baE:
  unimp
    "#
    .to_string()
}

fn process_statement(s: Statement) -> String {
    match &s {
        Statement::Label(l) => format!("{l}::\n"),
        Statement::Directive(_, _) => String::new(), // ignore
        Statement::Instruction(instr, args) => {
            let s = process_instruction(instr, args);
            assert!(s.ends_with('\n'));
            "  ".to_string() + &s[..s.len() - 1].replace('\n', "\n  ") + "\n"
        }
    }
}

fn to_number(x: &Argument) -> u32 {
    match x {
        Argument::Number(n) => *n as u32,
        Argument::HiDataRef(_) => 0, // TODO
        Argument::LoDataRef(_) => 0, // TODO
        Argument::Difference(_, _) => todo!(),
        _ => panic!(),
    }
}

fn rri(args: &[Argument]) -> (Register, Register, u32) {
    match args {
        [Argument::Register(r1), Argument::Register(r2), n] => (*r1, *r2, to_number(n)),
        _ => panic!(),
    }
}

fn rrr(args: &[Argument]) -> (Register, Register, Register) {
    match args {
        [Argument::Register(r1), Argument::Register(r2), Argument::Register(r3)] => (*r1, *r2, *r3),
        _ => panic!(),
    }
}

fn ri(args: &[Argument]) -> (Register, u32) {
    match args {
        [Argument::Register(r1), n] => (*r1, to_number(n)),
        _ => panic!(),
    }
}

fn rr(args: &[Argument]) -> (Register, Register) {
    match args {
        [Argument::Register(r1), Argument::Register(r2)] => (*r1, *r2),
        _ => panic!(),
    }
}

fn rrl(args: &[Argument]) -> (Register, Register, &str) {
    match args {
        [Argument::Register(r1), Argument::Register(r2), Argument::Symbol(l)] => (*r1, *r2, l),
        _ => panic!(),
    }
}

fn rl(args: &[Argument]) -> (Register, &str) {
    match args {
        [Argument::Register(r1), Argument::Symbol(l)] => (*r1, l),
        _ => panic!(),
    }
}

fn rro(args: &[Argument]) -> (Register, Register, u32) {
    match args {
        [Argument::Register(r1), Argument::RegOffset(r2, off)] => (*r1, *r2, *off as u32),
        _ => panic!(),
    }
}

fn process_instruction(instr: &str, args: &[Argument]) -> String {
    match instr {
        "add" => {
            let (rd, r1, r2) = rrr(args);
            format!("{rd} <=X= wrap {r1} + {r2}\n")
        }
        "addi" => {
            let (rd, rs, imm) = rri(args);
            format!("{rd} <=X= wrap {rs} + {imm}\n")
        }
        "beq" => {
            let (r1, r2, label) = rrl(args);
            format!("branch_if_zero {r1} - {r2}, {label}\n")
        }
        "beqz" => {
            let (r1, label) = rl(args);
            format!("branch_if_zero {r1}, {label}\n")
        }
        "bgeu" => {
            let (r1, r2, label) = rrl(args);
            format!("branch_if_positive {r1} - {r2}, {label}\n")
        }
        "bltu" => {
            let (r1, r2, label) = rrl(args);
            format!("branch_if_positive {r2} - {r1}, {label}\n")
        }
        "bne" => {
            let (r1, r2, label) = rrl(args);
            format!("branch_if_nonzero {r1} - {r2}, {label}\n")
        }
        "bnez" => {
            let (r1, label) = rl(args);
            format!("branch_if_nonzero {r1}, {label}\n")
        }
        "j" => {
            if let [Argument::Symbol(label)] = args {
                format!("jump {label}\n")
            } else {
                panic!()
            }
        }
        "call" => {
            if let [Argument::Symbol(label)] = args {
                format!("call {label}\n")
            } else {
                panic!()
            }
        }
        "ecall" => {
            assert!(args.is_empty());
            "x10 <= ${ }\n".to_string()
        }
        "li" => {
            let (rd, imm) = ri(args);
            format!("{rd} <=X= {imm}\n")
        }
        "lui" => {
            let (rd, imm) = ri(args);
            format!("{rd} <=X= {}\n", imm << 12)
        }
        "lw" => {
            let (rd, rs, off) = rro(args);
            format!("addr <=X= wrap {rs} + {off}\n") + &format!("{rd} <=X= mload\n")
        }
        "sw" => {
            let (r1, r2, off) = rro(args);
            format!("addr <=X= wrap {r2} + {off}\n") + &format!("mstore {r1}\n")
        }
        "mv" => {
            let (rd, rs) = rr(args);
            format!("{rd} <=X= {rs}\n")
        }
        "ret" => {
            assert!(args.is_empty());
            "ret\n".to_string()
        }
        "seqz" => {
            let (rd, rs) = rr(args);
            format!("{rd} <=Y= is_equal_zero {rs}\n")
        }
        "slli" => {
            let (rd, rs, amount) = rri(args);
            assert!(amount <= 31);
            if amount <= 16 {
                format!("{rd} <=Y= wrap16 {rs} * {}\n", 1 << amount)
            } else {
                todo!();
            }
        }
        "unimp" => "fail\n".to_string(),
        "xor" => {
            let (rd, r1, r2) = rrr(args);
            format!("{rd} <=X= xor {r1}, {r2}\n")
        }
        _ => todo!("Unknown instruction: {instr}"),
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn simple_decoding() {
//         let data = [0x17u8, 0x03, 0x00, 0x00, 0x67, 0x00, 0x03, 0x00];
//         decode_section(".text", &data);
//     }

//     #[test]
//     fn complex_decoding() {
//         let data: [u8; 60] = [
//             0x41, 0x11, 0x06, 0xc6, 0x22, 0xc4, 0x26, 0xc2, 0x2e, 0x84, 0xaa, 0x84, 0x91, 0xc9,
//             0x93, 0x05, 0xd4, 0xff, 0x26, 0x85, 0x97, 0x00, 0x00, 0x00, 0xe7, 0x80, 0x00, 0x00,
//             0x0d, 0x05, 0x11, 0xa0, 0x0d, 0x45, 0xb3, 0x85, 0x84, 0x00, 0x2e, 0x95, 0xb7, 0xc5,
//             0xad, 0xde, 0x93, 0x85, 0xf5, 0xee, 0x2e, 0x95, 0xb2, 0x40, 0x22, 0x44, 0x92, 0x44,
//             0x41, 0x01, 0x82, 0x80,
//         ];

//         decode_section(".text", &data);
//     }
// }
