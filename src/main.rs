use std::process::ExitCode;

enum LexState {
    Def,
    Plus,
    Minus,
}

mod lexer {
    use std::process::ExitCode;

    #[derive(Debug, PartialEq, Eq)]
    pub enum Token {
        Num(u64),
        Plus,
        Minus,
        Mult,
        Div,
        ParL,
        ParR,
    }

    pub fn lex(input: &str) -> Result<Vec<Token>, String> {
        let mut v = Vec::<Token>::new();
        let mut pos = 0;
        let input = input.as_bytes();
        while pos < input.len() {
            match input[pos] {
                c if c.is_ascii_digit() => {
                    let start = pos;
                    pos += 1;
                    while pos < input.len() && input[pos].is_ascii_digit() {
                        pos += 1;
                    }
                    let num: u64 = std::str::from_utf8(&input[start..pos])
                        .unwrap()
                        .parse()
                        .unwrap();
                    v.push(Token::Num(num));
                }
                c if c.is_ascii_whitespace() => {
                    pos += 1;
                }
                b'+' => {
                    v.push(Token::Plus);
                    pos += 1;
                }
                b'-' => {
                    v.push(Token::Minus);
                    pos += 1;
                }
                b'*' => {
                    v.push(Token::Mult);
                    pos += 1;
                }
                b'/' => {
                    v.push(Token::Div);
                    pos += 1;
                }
                b'(' => {
                    v.push(Token::ParL);
                    pos += 1;
                }
                b')' => {
                    v.push(Token::ParR);
                    pos += 1;
                }
                _ => {
                    eprintln!("Invalid Character at {}", pos);
                    return Err(format!("Invalid Character at {}", pos));
                }
            }
        }

        Ok(v)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn lex_test() {
            let input = "3 + (12 - 8)/12*9-21 + 4* 3";
            let input2 = "3a+10-0=0";
            let expected = vec![
                Token::Num(3),
                Token::Plus,
                Token::ParL,
                Token::Num(12),
                Token::Minus,
                Token::Num(8),
                Token::ParR,
                Token::Div,
                Token::Num(12),
                Token::Mult,
                Token::Num(9),
                Token::Minus,
                Token::Num(21),
                Token::Plus,
                Token::Num(4),
                Token::Mult,
                Token::Num(3),
            ];
            assert_eq!(lex(input), Ok(expected));
            assert_eq!(lex(input2), Err(String::from("Invalid Character at 1")));
        }
    }
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 2 {
        eprintln!("Error: invalid argument count.");
        return ExitCode::FAILURE;
    }

    let input = argv[1].as_bytes();

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
