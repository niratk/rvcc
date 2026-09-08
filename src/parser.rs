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
            Ok(Node::Num(*n))
        }
        Some(Token::ParL) => {
            *pos += 1;
            let expr = parse_add(input, pos)?;
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
        ];
        let expected = Node::Add(
            Box::new(Node::Sub(
                Box::new(Node::Add(
                    Box::new(Node::Num(3)),
                    Box::new(Node::Mul(
                        Box::new(Node::Div(
                            Box::new(Node::Sub(Box::new(Node::Num(12)), Box::new(Node::Num(8)))),
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
        assert!(parse(&[]).is_err());
    }

    #[test]
    fn parse_trailing_operator() {
        let input = vec![Token::Num(1), Token::Plus];
        assert!(parse(&input).is_err());
    }

    #[test]
    fn parse_consecutive_operators() {
        let input = vec![Token::Num(1), Token::Plus, Token::Mult, Token::Num(2)];
        assert!(parse(&input).is_err());
    }

    #[test]
    fn parse_missing_right_parenthesis() {
        let input = vec![Token::ParL, Token::Num(1), Token::Plus, Token::Num(2)];
        assert!(parse(&input).is_err());
    }

    #[test]
    fn parse_extra_right_parenthesis() {
        let input = vec![Token::Num(1), Token::Plus, Token::Num(2), Token::ParR];
        assert!(parse(&input).is_err());
    }

    #[test]
    fn parse_empty_parentheses() {
        let input = vec![Token::ParL, Token::ParR];
        assert!(parse(&input).is_err());
    }

    #[test]
    fn parse_missing_operator() {
        let input = vec![Token::Num(1), Token::Num(2)];
        assert!(parse(&input).is_err());
    }

    #[test]
    fn parse_double_division() {
        let input = vec![Token::Num(1), Token::Div, Token::Div, Token::Num(2)];
        assert!(parse(&input).is_err());
    }
}
