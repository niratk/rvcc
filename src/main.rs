use std::process::ExitCode;

mod codegen;
mod lexer;
mod parser;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 2 {
        eprintln!("Error: invalid argument count.");
        return ExitCode::FAILURE;
    }

    let input = &argv[1];

    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    codegen::generate(&ast);

    ExitCode::SUCCESS
}
