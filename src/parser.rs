use std::str::FromStr;

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
    Assign(Box<Node>, Box<Node>),
    Prog(Vec<Node>),
    Id(String),
    Num(u64),
    Return(Box<Node>),
    If(Box<Node>, Box<Node>),
    IfElse(Box<Node>, Box<Node>, Box<Node>),
    While(Box<Node>, Box<Node>),
    For(
        Option<Box<Node>>,
        Option<Box<Node>>,
        Option<Box<Node>>,
        Box<Node>,
    ),
}

fn parse_factor(input: &[Token], pos: &mut usize) -> Result<Node, String> {
    match input.get(*pos) {
        Some(Token::Num(n)) => {
            *pos += 1;
            Ok(Node::Num(*n))
        }
        Some(Token::Id(id)) => {
            *pos += 1;
            Ok(Node::Id(id.clone()))
        }
        Some(Token::ParL) => {
            *pos += 1;
            let expr = parse_expr(input, pos)?;
            match input.get(*pos) {
                Some(Token::ParR) => {
                    *pos += 1;
                    Ok(expr)
                }
                None => Err(String::from("Unexpected EOF.")),
                _ => Err(String::from("Unexpected character. ')' is expected.")),
            }
        }
        None => Err(String::from("Unexpected EOF")),
        Some(token) => Err(format!("Unexpected token: {:?}", token)),
    }
}

fn parse_unary(input: &[Token], pos: &mut usize) -> Result<Node, String> {
    match input.get(*pos) {
        Some(Token::Plus) => {
            *pos += 1;
            parse_factor(input, pos)
        }
        Some(Token::Minus) => {
            *pos += 1;
            let factor = parse_factor(input, pos)?;
            Ok(Node::Sub(Box::new(Node::Num(0)), Box::new(factor)))
        }
        _ => parse_factor(input, pos),
    }
}

fn parse_mul(input: &[Token], pos: &mut usize) -> Result<Node, String> {
    let mut lhs = parse_unary(input, pos)?;
    loop {
        match input.get(*pos) {
            Some(Token::Mult) => {
                *pos += 1;
                let rhs = parse_unary(input, pos)?;
                lhs = Node::Mul(Box::new(lhs), Box::new(rhs));
            }
            Some(Token::Div) => {
                *pos += 1;
                let rhs = parse_unary(input, pos)?;
                lhs = Node::Div(Box::new(lhs), Box::new(rhs));
            }
            _ => break,
        }
    }
    Ok(lhs)
}

fn parse_add(input: &[Token], pos: &mut usize) -> Result<Node, String> {
    let mut lhs = parse_mul(input, pos)?;
    loop {
        match input.get(*pos) {
            Some(Token::Plus) => {
                *pos += 1;
                let rhs = parse_mul(input, pos)?;
                lhs = Node::Add(Box::new(lhs), Box::new(rhs));
            }
            Some(Token::Minus) => {
                *pos += 1;
                let rhs = parse_mul(input, pos)?;
                lhs = Node::Sub(Box::new(lhs), Box::new(rhs));
            }
            _ => break,
        }
    }
    Ok(lhs)
}

fn parse_cmp(input: &[Token], pos: &mut usize) -> Result<Node, String> {
    let mut lhs = parse_add(input, pos)?;
    loop {
        match input.get(*pos) {
            Some(Token::Geq) => {
                *pos += 1;
                let rhs = parse_add(input, pos)?;
                lhs = Node::Lt(Box::new(rhs), Box::new(lhs));
            }
            Some(Token::Gt) => {
                *pos += 1;
                let rhs = parse_add(input, pos)?;
                lhs = Node::Leq(Box::new(rhs), Box::new(lhs));
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
            _ => break,
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
            _ => break,
        }
    }
    Ok(lhs)
}

fn parse_assign(input: &[Token], pos: &mut usize) -> Result<Node, String> {
    let lhs = parse_eq(input, pos)?;
    match input.get(*pos) {
        Some(Token::Assign) => {
            if let Node::Id(_) = lhs {
                *pos += 1;
                let rhs = parse_assign(input, pos)?;
                return Ok(Node::Assign(Box::new(lhs), Box::new(rhs)));
            } else {
                return Err(String::from("lhs of assign stmt must be identifier."));
            }
        }
        _ => {
            return Ok(lhs);
        }
    }
}

fn parse_expr(input: &[Token], pos: &mut usize) -> Result<Node, String> {
    parse_assign(input, pos)
}

fn parse_stmt(input: &[Token], pos: &mut usize) -> Result<Node, String> {
    match input.get(*pos) {
        Some(Token::If) => {
            *pos += 1;
            match input.get(*pos) {
                Some(Token::ParL) => {
                    *pos += 1;
                }
                t => {
                    return Err(format!("unexpected token: {:?}", t));
                }
            }
            let cond = parse_expr(input, pos)?;
            match input.get(*pos) {
                Some(Token::ParR) => {
                    *pos += 1;
                }
                t => {
                    return Err(format!("unexpected token: {:?}", t));
                }
            }
            let stmt = parse_stmt(input, pos)?;
            match input.get(*pos) {
                Some(Token::Else) => {
                    *pos += 1;
                }
                _ => {
                    return Ok(Node::If(Box::new(cond), Box::new(stmt)));
                }
            }
            let stmt2 = parse_stmt(input, pos)?;
            Ok(Node::IfElse(
                Box::new(cond),
                Box::new(stmt),
                Box::new(stmt2),
            ))
        }
        Some(Token::While) => {
            *pos += 1;
            match input.get(*pos) {
                Some(Token::ParL) => {
                    *pos += 1;
                }
                t => {
                    return Err(format!("unexpected token: {:?}", t));
                }
            }
            let cond = parse_expr(input, pos)?;
            match input.get(*pos) {
                Some(Token::ParR) => {
                    *pos += 1;
                }
                t => {
                    return Err(format!("unexpected token {:?}", t));
                }
            }
            let stmt = parse_stmt(input, pos)?;
            Ok(Node::While(Box::new(cond), Box::new(stmt)))
        }
        Some(Token::For) => {
            *pos += 1;
            match input.get(*pos) {
                Some(Token::ParL) => {
                    *pos += 1;
                }
                t => {
                    return Err(format!("unexpected token {:?}", t));
                }
            }
            let mut e1 = None;
            match input.get(*pos) {
                Some(Token::Semi) => {
                    *pos += 1;
                }
                _ => {
                    e1 = Some(Box::new(parse_expr(input, pos)?));
                    match input.get(*pos) {
                        Some(Token::Semi) => {
                            *pos += 1;
                        }
                        t => {
                            return Err(format!("unexpected token {:?}", t));
                        }
                    }
                }
            }
            let mut e2 = None;
            match input.get(*pos) {
                Some(Token::Semi) => {
                    *pos += 1;
                }
                _ => {
                    e2 = Some(Box::new(parse_expr(input, pos)?));
                    match input.get(*pos) {
                        Some(Token::Semi) => {
                            *pos += 1;
                        }
                        t => {
                            return Err(format!("unexpected token {:?}", t));
                        }
                    }
                }
            }
            let mut e3 = None;
            match input.get(*pos) {
                Some(Token::ParR) => {
                    *pos += 1;
                }
                _ => {
                    e3 = Some(Box::new(parse_expr(input, pos)?));
                    match input.get(*pos) {
                        Some(Token::ParR) => {
                            *pos += 1;
                        }
                        t => {
                            return Err(format!("unexpected token {:?}", t));
                        }
                    }
                }
            }
            let stmt = parse_stmt(input, pos)?;

            Ok(Node::For(e1, e2, e3, Box::new(stmt)))
        }
        Some(Token::Return) => {
            *pos += 1;
            let expr = parse_expr(input, pos)?;

            if !matches!(input.get(*pos), Some(Token::Semi)) {
                return Err(String::from("';' is required at the end of statements."));
            }
            *pos += 1;

            Ok(Node::Return(Box::new(expr)))
        }
        _ => {
            let expr = parse_expr(input, pos)?;
            if !matches!(input.get(*pos), Some(Token::Semi)) {
                return Err(String::from("';' is required at the end of statements."));
            }
            *pos += 1;
            Ok(expr)
        }
    }
}

fn parse_prog(input: &[Token], pos: &mut usize) -> Result<Node, String> {
    let mut prog: Vec<Node> = vec![];
    loop {
        let stmt = parse_stmt(input, pos)?;
        prog.push(stmt);
        if *pos == input.len() {
            break;
        }
    }

    Ok(Node::Prog(prog))
}

pub fn parse(input: &[Token]) -> Result<Node, String> {
    let mut pos = 0;
    let node = parse_prog(input, &mut pos)?;

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

    fn parse_source(input: &str) -> Node {
        let tokens = crate::lexer::lex(input).unwrap();
        parse(&tokens).unwrap()
    }

    #[test]
    fn parse_test() {
        let input = vec![
            Token::Id(String::from("a")),
            Token::Assign,
            Token::Num(3),
            Token::Plus,
            Token::Num(2),
            Token::Geq,
            Token::Num(4),
            Token::Semi,
            Token::Num(6),
            Token::Div,
            Token::Id(String::from("a")),
            Token::Semi,
            Token::Return,
            Token::Id(String::from("a")),
            Token::Semi,
        ];
        let expected = Node::Prog(vec![
            Node::Assign(
                Box::new(Node::Id(String::from("a"))),
                Box::new(Node::Lt(
                    Box::new(Node::Num(4)),
                    Box::new(Node::Add(Box::new(Node::Num(3)), Box::new(Node::Num(2)))),
                )),
            ),
            Node::Div(
                Box::new(Node::Num(6)),
                Box::new(Node::Id(String::from("a"))),
            ),
            Node::Return(Box::new(Node::Id(String::from("a")))),
        ]);

        assert_eq!(parse(&input).unwrap(), expected);
    }

    #[test]
    fn parse_if_statement() {
        let expected = Node::Prog(vec![Node::If(
            Box::new(Node::Id(String::from("condition"))),
            Box::new(Node::Return(Box::new(Node::Num(1)))),
        )]);

        assert_eq!(parse_source("if (condition) return 1;"), expected);
    }

    #[test]
    fn parse_if_else_statement() {
        let expected = Node::Prog(vec![Node::IfElse(
            Box::new(Node::Id(String::from("condition"))),
            Box::new(Node::Return(Box::new(Node::Num(1)))),
            Box::new(Node::Return(Box::new(Node::Num(2)))),
        )]);

        assert_eq!(
            parse_source("if (condition) return 1; else return 2;"),
            expected
        );
    }

    #[test]
    fn parse_else_with_nearest_if_statement() {
        let expected = Node::Prog(vec![Node::If(
            Box::new(Node::Id(String::from("a"))),
            Box::new(Node::IfElse(
                Box::new(Node::Id(String::from("b"))),
                Box::new(Node::Return(Box::new(Node::Num(1)))),
                Box::new(Node::Return(Box::new(Node::Num(2)))),
            )),
        )]);

        assert_eq!(
            parse_source("if (a) if (b) return 1; else return 2;"),
            expected
        );
    }

    #[test]
    fn parse_while_statement() {
        let expected = Node::Prog(vec![Node::While(
            Box::new(Node::Lt(
                Box::new(Node::Id(String::from("i"))),
                Box::new(Node::Num(10)),
            )),
            Box::new(Node::Assign(
                Box::new(Node::Id(String::from("i"))),
                Box::new(Node::Add(
                    Box::new(Node::Id(String::from("i"))),
                    Box::new(Node::Num(1)),
                )),
            )),
        )]);

        assert_eq!(parse_source("while (i < 10) i = i + 1;"), expected);
    }

    #[test]
    fn parse_for_statement() {
        let expected = Node::Prog(vec![Node::For(
            Some(Box::new(Node::Assign(
                Box::new(Node::Id(String::from("i"))),
                Box::new(Node::Num(0)),
            ))),
            Some(Box::new(Node::Lt(
                Box::new(Node::Id(String::from("i"))),
                Box::new(Node::Num(10)),
            ))),
            Some(Box::new(Node::Assign(
                Box::new(Node::Id(String::from("i"))),
                Box::new(Node::Add(
                    Box::new(Node::Id(String::from("i"))),
                    Box::new(Node::Num(1)),
                )),
            ))),
            Box::new(Node::Assign(
                Box::new(Node::Id(String::from("sum"))),
                Box::new(Node::Add(
                    Box::new(Node::Id(String::from("sum"))),
                    Box::new(Node::Id(String::from("i"))),
                )),
            )),
        )]);

        assert_eq!(
            parse_source("for (i = 0; i < 10; i = i + 1) sum = sum + i;"),
            expected
        );
    }

    #[test]
    fn parse_for_statement_with_omitted_expressions() {
        let expected = Node::Prog(vec![Node::For(
            None,
            None,
            None,
            Box::new(Node::Return(Box::new(Node::Num(0)))),
        )]);

        assert_eq!(parse_source("for (;;) return 0;"), expected);
    }
}
