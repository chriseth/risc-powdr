use std::fmt::{self, Display};

use lalrpop_util::*;

lalrpop_mod!(
    #[allow(clippy::all)]
    riscv_asm
);

pub enum Statement {
    Label(String),
    Directive(String, Vec<Argument>),
    Instruction(String, Vec<Argument>),
}
pub enum Argument {
    Register(Register),
    Number(i64),
    RegOffset(Register, i64),
    StringLiteral(String),
    Symbol(String),
    HiDataRef(String),
    LoDataRef(String),
    Difference(String, String),
}

#[derive(Clone, Copy)]
pub struct Register(u8);

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Label(l) => writeln!(f, "{l}:"),
            Statement::Directive(d, args) => writeln!(f, "  .{d} {}", format_arguments(args)),
            Statement::Instruction(i, args) => writeln!(f, "  {i} {}", format_arguments(args)),
        }
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Argument::Register(r) => write!(f, "{r}"),
            Argument::Number(n) => write!(f, "{n}"),
            Argument::RegOffset(reg, off) => write!(f, "{off}({reg})"),
            Argument::StringLiteral(lit) => write!(f, "\"{lit}\""),
            Argument::Symbol(s) => write!(f, "{s}"),
            Argument::HiDataRef(sym) => write!(f, "%hi({sym})"),
            Argument::LoDataRef(sym) => write!(f, "%lo({sym})"),
            Argument::Difference(left, right) => write!(f, "{left} - {right}"),
        }
    }
}

fn format_arguments(args: &[Argument]) -> String {
    args.iter()
        .map(|a| format!("{a}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x{}", self.0)
    }
}

pub fn parse_asm(input: &str) -> Vec<Statement> {
    input
        .split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .flat_map(|line| {
            riscv_asm::MaybeStatementParser::new().parse(line).unwrap()
            //.map_err(|err| handle_error(err, file_name, input))
        })
        .collect()
}
