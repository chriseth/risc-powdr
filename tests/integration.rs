use mktemp::Temp;
use std::fs;
use std::process::Command;

#[test]
fn test_sum() {
    let case = "sum.rs";
    let asm = compile_to_asm(case);
    risc_powdr::compiler::compile_riscv_asm(&asm);
}

fn compile_to_asm(test_case: &str) -> String {
    let temp_dir = Temp::new_dir().unwrap();
    let obj_file_path = format!("{}/out.asm", temp_dir.to_str().unwrap());
    let rustc_status = Command::new("rustc")
        .args([
            "--target",
            "riscv32imc-unknown-none-elf",
            "-C",
            "target-feature=-c",
            "--crate-type",
            "lib",
            "--emit=asm",
            "-C",
            "opt-level=3",
            "-C",
            "embed-bitcode=no",
            "-C",
            "debuginfo=0",
            "-o",
            &obj_file_path,
            &format!("tests/cases/{test_case}"),
        ])
        .status()
        .unwrap();
    assert!(rustc_status.success());
    fs::read_to_string(obj_file_path).unwrap()
}
