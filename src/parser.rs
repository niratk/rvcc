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
    let is_return_stmt = matches!(input.get(*pos), Some(Token::Return));
    if is_return_stmt {
        *pos += 1;
    }

    let expr = parse_expr(input, pos)?;

    if !matches!(input.get(*pos), Some(Token::Semi)) {
        return Err(String::from("';' is required at the end of statement."));
    }
    *pos += 1;

    Ok(if is_return_stmt {
        Node::Return(Box::new(expr))
    } else {
        expr
    })
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
}
