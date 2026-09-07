use std::process::ExitCode;

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
        Eq,
        Neq,
        Gt,
        Geq,
        Lt,
        Leq,
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
                b'>' => match input.get(pos + 1) {
                    Some(b'=') => {
                        v.push(Token::Geq);
                        pos += 2;
                    }
                    _ => {
                        v.push(Token::Gt);
                        pos += 1;
                    }
                },
                b'<' => match input.get(pos + 1) {
                    Some(b'=') => {
                        v.push(Token::Leq);
                        pos += 2;
                    }
                    _ => {
                        v.push(Token::Lt);
                        pos += 1;
                    }
                },
                b'=' => match input.get(pos + 1) {
                    Some(b'=') => {
                        v.push(Token::Eq);
                        pos += 2;
                    }
                    _ => {
                        eprintln!("Invalid Character at {}", pos);
                        return Err(format!("Invalid Character at {}", pos));
                    }
                },
                b'!' => match input.get(pos + 1) {
                    Some(b'=') => {
                        v.push(Token::Neq);
                        pos += 2;
                    }
                    _ => {
                        eprintln!("Invalid Character at {}", pos);
                        return Err(format!("Invalid Character at {}", pos));
                    }
                },
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
        Leq(Box<Node>, Box<Node>),
        Lt(Box<Node>, Box<Node>),
        Eq(Box<Node>, Box<Node>),
        Neq(Box<Node>, Box<Node>),
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
                let e = parse_add(input, pos)?;
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

    fn parse_unary(input: &[Token], pos: &mut usize) -> Result<Node, String> {
        match input.get(*pos) {
            Some(Token::Plus) => {
                *pos += 1;
                return parse_factor(input, pos);
            }
            Some(Token::Minus) => {
                *pos += 1;
                let f = parse_factor(input, pos)?;
                return Ok(Node::Sub(Box::new(Node::Num(0)), Box::new(f)));
            }
            _ => {
                return parse_factor(input, pos);
            }
        }
    }

    fn parse_mul(input: &[Token], pos: &mut usize) -> Result<Node, String> {
        let mut f = parse_unary(input, pos)?;
        loop {
            match input.get(*pos) {
                Some(Token::Mult) => {
                    *pos += 1;
                    let f2 = parse_unary(input, pos)?;
                    f = Node::Mul(Box::new(f), Box::new(f2));
                }
                Some(Token::Div) => {
                    *pos += 1;
                    let f2 = parse_unary(input, pos)?;
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

    fn parse_add(input: &[Token], pos: &mut usize) -> Result<Node, String> {
        let mut t = parse_mul(input, pos)?;
        loop {
            match input.get(*pos) {
                Some(Token::Plus) => {
                    *pos += 1;
                    let t2 = parse_mul(input, pos)?;
                    t = Node::Add(Box::new(t), Box::new(t2));
                }
                Some(Token::Minus) => {
                    *pos += 1;
                    let t2 = parse_mul(input, pos)?;
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

    fn parse_cmp(input: &[Token], pos: &mut usize) -> Result<Node, String> {
        let mut lhs = parse_add(input, pos)?;
        loop {
            match input.get(*pos) {
                Some(Token::Geq) => {
                    *pos += 1;
                    let rhs = parse_add(input, pos)?;
                    lhs = Node::Leq(Box::new(rhs), Box::new(lhs));
                }
                Some(Token::Gt) => {
                    *pos += 1;
                    let rhs = parse_add(input, pos)?;
                    lhs = Node::Lt(Box::new(rhs), Box::new(lhs));
                }
                Some(Token::Leq) => {
                    *pos += 1;
                    let rhs = parse_add(input, pos)?;
                    lhs = Node::Leq(Box::new(lhs), Box::new(rhs));
                }
                Some(Token::Lt) => {
                    *pos += 1;
                    let rhs = parse_add(input, pos)?;
                    lhs = Node::Lt(Box::new(lhs), Box::new(rhs));
                }
                _ => {
                    break;
                }
            }
        }

        Ok(lhs)
    }

    fn parse_eq(input: &[Token], pos: &mut usize) -> Result<Node, String> {
        let mut lhs = parse_cmp(input, pos)?;
        loop {
            match input.get(*pos) {
                Some(Token::Eq) => {
                    *pos += 1;
                    let rhs = parse_cmp(input, pos)?;
                    lhs = Node::Eq(Box::new(lhs), Box::new(rhs));
                }
                Some(Token::Neq) => {
                    *pos += 1;
                    let rhs = parse_cmp(input, pos)?;
                    lhs = Node::Neq(Box::new(lhs), Box::new(rhs));
                }
                _ => {
                    break;
                }
            }
        }

        Ok(lhs)
    }

    fn parse_expr(input: &[Token], pos: &mut usize) -> Result<Node, String> {
        parse_eq(input, pos)
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
        fn parse_gt() {
            let input = vec![Token::Num(2), Token::Lt, Token::Num(3)];

            let expected = Node::Lt(Box::new(Node::Num(2)), Box::new(Node::Num(3)));

            assert_eq!(expected, parse(&input).unwrap());
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

mod codegen {
    // generate stack machine code. THe code is generated in preorder of AST.

    use crate::parser::Node;

    pub fn generate(ast: &Node) {
        match ast {
            Node::Add(l, r) => {
                generate(l);
                generate(r);
                println!("ld t1,0(sp)");
                println!("addi sp,sp,16");
                println!("ld t0,0(sp)");
                println!("add t0,t0,t1");
                println!("sd t0,0(sp)");
            }
            Node::Sub(l, r) => {
                generate(l);
                generate(r);
                println!("ld t1,0(sp)");
                println!("addi sp,sp,16");
                println!("ld t0,0(sp)");
                println!("sub t0,t0,t1");
                println!("sd t0,0(sp)");
            }
            Node::Mul(l, r) => {
                // target=RV64M
                generate(l);
                generate(r);
                println!("ld t1,0(sp)");
                println!("addi sp,sp,16");
                println!("ld t0,0(sp)");
                println!("mul t0,t0,t1");
                println!("sd t0,0(sp)");
            }
            Node::Div(l, r) => {
                // target=RV64M
                generate(l);
                generate(r);
                println!("ld t1,0(sp)");
                println!("addi sp,sp,16");
                println!("ld t0,0(sp)");
                println!("div t0,t0,t1");
                println!("sd t0,0(sp)");
            }
            Node::Lt(l, r) => {
                generate(l);
                generate(r);
                println!("ld t1,0(sp)");
                println!("addi sp,sp,16");
                println!("ld t0,0(sp)");
                println!("slt t0,t0,t1");
                println!("sd t0,0(sp)");
            }
            Node::Leq(l, r) => {
                generate(l);
                generate(r);
                println!("ld t1,0(sp)");
                println!("addi sp,sp,16");
                println!("ld t0,0(sp)");
                println!("slt t0,t1,t0");
                println!("xori t0,t0,1");
                println!("sd t0,0(sp)");
            }
            Node::Eq(l, r) => {
                generate(l);
                generate(r);
                println!("ld t1,0(sp)");
                println!("addi sp,sp,16");
                println!("ld t0,0(sp)");
                println!("sub t0,t0,t1");
                println!("seqz t0,t0");
                println!("sd t0,0(sp)");
            }
            Node::Neq(l, r) => {
                generate(l);
                generate(r);
                println!("ld t1,0(sp)");
                println!("addi sp,sp,16");
                println!("ld t0,0(sp)");
                println!("sub t0,t0,t1");
                println!("snez t0,t0");
                println!("sd t0,0(sp)");
            }
            Node::Num(n) => {
                // push n
                println!("addi sp,sp,-16");
                println!("li t0,{}", n);
                println!("sd t0,0(sp)");
            }
        }
    }
}

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

    let l = lexer::lex(input).unwrap();
    let ast = parser::parse(&l).unwrap();
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
