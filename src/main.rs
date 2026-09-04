use std::process::ExitCode;

enum LexState {
    Def,
    Plus,
    Minus,
}

mod lexer {

    #[derive(Debug, PartialEq, Eq, Clone)]
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

mod parser {
    use crate::lexer::Token;

    #[derive(Debug, PartialEq, Eq)]
    pub enum Node {
        Add(Box<Node>, Box<Node>),
        Sub(Box<Node>, Box<Node>),
        Mul(Box<Node>, Box<Node>),
        Div(Box<Node>, Box<Node>),
        Num(u64),
    }

    fn parse_factor(input: &[Token], pos: &mut usize) -> Result<Node, String> {
        match input.get(*pos) {
            Some(Token::Num(n)) => {
                *pos += 1;
                return Ok(Node::Num(*n));
            }
            Some(Token::ParL) => {
                *pos += 1;
                let e = parse_expr(input, pos)?;
                match input.get(*pos) {
                    Some(Token::ParR) => {
                        *pos += 1;
                        return Ok(e);
                    }
                    None => {
                        return Err(String::from("Unexpected EOF."));
                    }
                    _ => {
                        return Err(String::from("Unexpected character. ')' is expected."));
                    }
                }
            }
            None => {
                return Err(String::from("Unexpected EOF"));
            }
            Some(token) => {
                return Err(format!("Unexpected token: {:?}", token));
            }
        }
    }

    fn parse_term(input: &[Token], pos: &mut usize) -> Result<Node, String> {
        let mut f = parse_factor(input, pos)?;
        loop {
            match input.get(*pos) {
                Some(Token::Mult) => {
                    *pos += 1;
                    let f2 = parse_factor(input, pos)?;
                    f = Node::Mul(Box::new(f), Box::new(f2));
                }
                Some(Token::Div) => {
                    *pos += 1;
                    let f2 = parse_factor(input, pos)?;
                    f = Node::Div(Box::new(f), Box::new(f2));
                }
                None => {
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        Ok(f)
    }

    fn parse_expr(input: &[Token], pos: &mut usize) -> Result<Node, String> {
        let mut t = parse_term(input, pos)?;
        loop {
            match input.get(*pos) {
                Some(Token::Plus) => {
                    *pos += 1;
                    let t2 = parse_term(input, pos)?;
                    t = Node::Add(Box::new(t), Box::new(t2));
                }
                Some(Token::Minus) => {
                    *pos += 1;
                    let t2 = parse_term(input, pos)?;
                    t = Node::Sub(Box::new(t), Box::new(t2));
                }
                None => {
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        Ok(t)
    }

    pub fn parse(input: &[Token]) -> Result<Node, String> {
        let mut pos = 0;

        let node = parse_expr(input, &mut pos)?;

        if pos != input.len() {
            return Err(format!(
                "Unexpected token at position {}: {:?}",
                pos, input[pos]
            ));
        }

        Ok(node)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_test() {
            let input = vec![
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
            ]; // "3 + (12 - 8)/12*9-21 + 4* 3"
            let expected = Node::Add(
                Box::new(Node::Sub(
                    Box::new(Node::Add(
                        Box::new(Node::Num(3)),
                        Box::new(Node::Mul(
                            Box::new(Node::Div(
                                Box::new(Node::Sub(
                                    Box::new(Node::Num(12)),
                                    Box::new(Node::Num(8)),
                                )),
                                Box::new(Node::Num(12)),
                            )),
                            Box::new(Node::Num(9)),
                        )),
                    )),
                    Box::new(Node::Num(21)),
                )),
                Box::new(Node::Mul(Box::new(Node::Num(4)), Box::new(Node::Num(3)))),
            );
            assert_eq!(parse(&input), Ok(expected));
        }

        #[test]
        fn parse_empty_input() {
            let input = vec![];

            assert!(parse(&input).is_err());
        }

        #[test]
        fn parse_trailing_operator() {
            // "1 +"
            let input = vec![Token::Num(1), Token::Plus];

            assert!(parse(&input).is_err());
        }

        #[test]
        fn parse_leading_operator() {
            // "+ 1"
            let input = vec![Token::Plus, Token::Num(1)];

            assert!(parse(&input).is_err());
        }

        #[test]
        fn parse_consecutive_operators() {
            // "1 + * 2"
            let input = vec![Token::Num(1), Token::Plus, Token::Mult, Token::Num(2)];

            assert!(parse(&input).is_err());
        }

        #[test]
        fn parse_missing_right_parenthesis() {
            // "(1 + 2"
            let input = vec![Token::ParL, Token::Num(1), Token::Plus, Token::Num(2)];

            assert!(parse(&input).is_err());
        }

        #[test]
        fn parse_extra_right_parenthesis() {
            // "1 + 2)"
            let input = vec![Token::Num(1), Token::Plus, Token::Num(2), Token::ParR];

            assert!(parse(&input).is_err());
        }

        #[test]
        fn parse_empty_parentheses() {
            // "()"
            let input = vec![Token::ParL, Token::ParR];

            assert!(parse(&input).is_err());
        }

        #[test]
        fn parse_missing_operator() {
            // "1 2"
            let input = vec![Token::Num(1), Token::Num(2)];

            assert!(parse(&input).is_err());
        }

        #[test]
        fn parse_double_division() {
            // "1 / / 2"
            let input = vec![Token::Num(1), Token::Div, Token::Div, Token::Num(2)];

            assert!(parse(&input).is_err());
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
