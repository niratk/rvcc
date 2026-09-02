use std::process::ExitCode;

enum LexState {
    Def,
    Plus,
    Minus,
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 2 {
        eprintln!("Error: invalid argument count.");
        return ExitCode::FAILURE;
    }

    let mut input = argv[1].as_bytes();

    println!(".globl main");
    println!("main:");

    println!("addi sp,sp,-32");
    println!("sd ra,24(sp)");
    println!("sd s0,16(sp)");
    println!("addi s0,sp,32");
    println!("sw a0,-20(s0)");
    println!("sd a1,-32(s0)");

    println!("li a5, 0");

    let mut state = LexState::Def;

    let mut pos: usize = 0;
    while pos < input.len() {
        match input[pos] {
            b'+' => {
                pos += 1;
                state = LexState::Plus;
            }
            b'-' => {
                pos += 1;
                state = LexState::Minus;
            }
            b'0'..=b'9' => {
                let start = pos;
                pos += 1;
                while pos < input.len() && input[pos].is_ascii_digit() {
                    pos += 1;
                }
                let value: u64 = std::str::from_utf8(&input[start..pos])
                    .unwrap()
                    .parse()
                    .unwrap();
                match state {
                    LexState::Def => {
                        println!("li a5,{}", value);
                    }
                    LexState::Plus => {
                        println!("addi a5,a5,{}", value);
                    }
                    LexState::Minus => {
                        println!("addi a5,a5,-{}", value);
                    }
                }
            }
            _ => {
                eprintln!("Invalid Token Detected!");
                return ExitCode::FAILURE;
            }
        }
    }

    println!("mv a0,a5");
    println!("ld ra,24(sp)");
    println!("ld s0,16(sp)");
    println!("addi sp,sp,32");
    println!("jr ra");

    ExitCode::SUCCESS
}
