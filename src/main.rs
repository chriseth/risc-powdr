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
}

fn main() {
    match Cli::parse().command {
        Commands::Compile { input } => {
            risc_powdr::compiler::compile_file(Path::new(&input));
        }
    }
}
