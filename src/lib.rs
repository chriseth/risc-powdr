use std::process::Command;

use mktemp::Temp;
use std::fs;

pub mod asm_parser;
pub mod compiler;

pub fn compile_rust_to_asm(input_file: &str) -> String {
    let temp_file = Temp::new_file().unwrap();
    let rustc_status = Command::new("rustc")
        .args([
            "--target",
            "riscv32imc-unknown-none-elf",
            "--crate-type",
            "lib",
            "--emit=asm",
            "-C",
            "opt-level=3",
            "-o",
            temp_file.to_str().unwrap(),
            input_file,
        ])
        .status()
        .unwrap();
    assert!(rustc_status.success());
    fs::read_to_string(temp_file.to_str().unwrap()).unwrap()
}
