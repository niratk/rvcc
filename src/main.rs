use std::process::ExitCode;

mod ast;
mod codegen;
mod ir;
mod lexer;
mod parser;
mod sema;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 2 {
        eprintln!("Error: invalid argument count.");
        return ExitCode::FAILURE;
    }

    let input = &argv[1];

    let result = lexer::lex(input)
        .and_then(|tokens| parser::parse(&tokens))
        .and_then(|ast| sema::analyze(ast).map_err(|error| error.to_string()));

    let program = match result {
        Ok(program) => program,
        Err(error) => {
            eprintln!("Error: {error}");
            return ExitCode::FAILURE;
        }
    };
    print!("{}", codegen::generate(&program));

    ExitCode::SUCCESS
}
