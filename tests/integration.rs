use mktemp::Temp;
use powdr::number::AbstractNumberType;
use std::fs;
use std::process::Command;

#[test]
fn test_sum() {
    let case = "sum.rs";
    verify(
        case,
        [16, 4, 1, 2, 8, 5].iter().map(|&x| x.into()).collect(),
    );
}

fn verify(case: &str, inputs: Vec<AbstractNumberType>) {
    let asm = compile_to_asm(case);
    let powdr_asm_source = risc_powdr::compiler::compile_riscv_asm(&asm);

    let temp_dir = mktemp::Temp::new_dir().unwrap();
    powdr::compiler::compile_asm_string(
        &format!("{case}.asm"),
        &powdr_asm_source,
        inputs,
        &temp_dir,
        false,
        false,
    )

    // TODO actually verify the generated columns
}

fn compile_to_asm(test_case: &str) -> String {
    let temp_dir = Temp::new_dir().unwrap();
    let asm_file_path = format!("{}/out.asm", temp_dir.to_str().unwrap());
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
            &asm_file_path,
            &format!("tests/cases/{test_case}"),
        ])
        .status()
        .unwrap();
    assert!(rustc_status.success());
    fs::read_to_string(asm_file_path).unwrap()
}
