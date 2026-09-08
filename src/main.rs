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

    println!(".globl main");
    println!("main:");

    println!("addi sp,sp,-32");
    println!("sd ra,24(sp)");
    println!("sd s0,16(sp)");
    println!("addi s0,sp,32");
    println!("sw a0,-20(s0)");
    println!("sd a1,-32(s0)");

    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    codegen::generate(&ast);
    println!("ld a5,0(sp)");
    println!("addi sp,sp,16");

    println!("mv a0,a5");
    println!("ld ra,24(sp)");
    println!("ld s0,16(sp)");
    println!("addi sp,sp,32");
    println!("jr ra");

    ExitCode::SUCCESS
}
