use clap::{Parser, Subcommand};
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Converts a RISC-V assembly file (riscv32imc-unknown-none-elf) to powdr assembly.
    Compile {
        /// Input file
        input: String,
    },
    /// Converts a "no_std" Rust file to powdr assembly.
    CompileRs {
        /// Input file
        input: String,
    },
}

fn main() {
    match Cli::parse().command {
        Commands::Compile { input } => {
            risc_powdr::compiler::compile_file(Path::new(&input));
        }
        Commands::CompileRs { input } => {
            let powdr_asm = risc_powdr::compile_rust_to_asm(&input);
            let output = risc_powdr::compiler::compile_riscv_asm(&powdr_asm);
            println!("{output}");
        }
    }
}
