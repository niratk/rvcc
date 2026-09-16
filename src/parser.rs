use crate::{
    ast::{BinaryOp, Block, Expr, Function, Program, Stmt},
    lexer::Token,
};

fn parse_factor(input: &[Token], pos: &mut usize) -> Result<Expr, String> {
    match input.get(*pos) {
        Some(Token::Num(number)) => {
            *pos += 1;
            Ok(Expr::Number(*number))
        }
        Some(Token::Id(name)) => {
            let name = name.clone();
            *pos += 1;
            if !matches!(input.get(*pos), Some(Token::ParL)) {
                return Ok(Expr::Variable(name));
            }

            *pos += 1;
            let mut args = Vec::new();
            if matches!(input.get(*pos), Some(Token::ParR)) {
                *pos += 1;
                return Ok(Expr::Call { name, args });
            }

            loop {
                args.push(parse_expr(input, pos)?);
                match input.get(*pos) {
                    Some(Token::Comma) => *pos += 1,
                    Some(Token::ParR) => {
                        *pos += 1;
                        break;
                    }
                    token => return Err(format!("unexpected token: {token:?}")),
                }
            }
            Ok(Expr::Call { name, args })
        }
        Some(Token::ParL) => {
            *pos += 1;
            let expr = parse_expr(input, pos)?;
            if !matches!(input.get(*pos), Some(Token::ParR)) {
                return Err(String::from("Unexpected character. ')' is expected."));
            }
            *pos += 1;
            Ok(expr)
        }
        None => Err(String::from("Unexpected EOF")),
        Some(token) => Err(format!("Unexpected token: {token:?}")),
    }
}

fn parse_unary(input: &[Token], pos: &mut usize) -> Result<Expr, String> {
    match input.get(*pos) {
        Some(Token::Plus) => {
            *pos += 1;
            parse_factor(input, pos)
        }
        Some(Token::Minus) => {
            *pos += 1;
            Ok(Expr::Binary {
                op: BinaryOp::Sub,
                lhs: Box::new(Expr::Number(0)),
                rhs: Box::new(parse_factor(input, pos)?),
            })
        }
        _ => parse_factor(input, pos),
    }
}

fn binary(lhs: Expr, op: BinaryOp, rhs: Expr) -> Expr {
    Expr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn parse_mul(input: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut lhs = parse_unary(input, pos)?;
    loop {
        let op = match input.get(*pos) {
            Some(Token::Mult) => BinaryOp::Mul,
            Some(Token::Div) => BinaryOp::Div,
            _ => break,
        };
        *pos += 1;
        lhs = binary(lhs, op, parse_unary(input, pos)?);
    }
    Ok(lhs)
}

fn parse_add(input: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut lhs = parse_mul(input, pos)?;
    loop {
        let op = match input.get(*pos) {
            Some(Token::Plus) => BinaryOp::Add,
            Some(Token::Minus) => BinaryOp::Sub,
            _ => break,
        };
        *pos += 1;
        lhs = binary(lhs, op, parse_mul(input, pos)?);
    }
    Ok(lhs)
}

fn parse_cmp(input: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut lhs = parse_add(input, pos)?;
    loop {
        match input.get(*pos) {
            Some(Token::Gt) => {
                *pos += 1;
                lhs = binary(parse_add(input, pos)?, BinaryOp::Less, lhs);
            }
            Some(Token::Geq) => {
                *pos += 1;
                lhs = binary(parse_add(input, pos)?, BinaryOp::LessEqual, lhs);
            }
            Some(Token::Lt) => {
                *pos += 1;
                lhs = binary(lhs, BinaryOp::Less, parse_add(input, pos)?);
            }
            Some(Token::Leq) => {
                *pos += 1;
                lhs = binary(lhs, BinaryOp::LessEqual, parse_add(input, pos)?);
            }
            _ => break,
        }
    }
    Ok(lhs)
}

fn parse_eq(input: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let mut lhs = parse_cmp(input, pos)?;
    loop {
        let op = match input.get(*pos) {
            Some(Token::Eq) => BinaryOp::Equal,
            Some(Token::Neq) => BinaryOp::NotEqual,
            _ => break,
        };
        *pos += 1;
        lhs = binary(lhs, op, parse_cmp(input, pos)?);
    }
    Ok(lhs)
}

fn parse_assign(input: &[Token], pos: &mut usize) -> Result<Expr, String> {
    let lhs = parse_eq(input, pos)?;
    if !matches!(input.get(*pos), Some(Token::Assign)) {
        return Ok(lhs);
    }
    let Expr::Variable(name) = lhs else {
        return Err(String::from("lhs of assign expression must be identifier."));
    };
    *pos += 1;
    Ok(Expr::Assign {
        name,
        value: Box::new(parse_assign(input, pos)?),
    })
}

fn parse_expr(input: &[Token], pos: &mut usize) -> Result<Expr, String> {
    parse_assign(input, pos)
}

fn parse_block(input: &[Token], pos: &mut usize) -> Result<Block, String> {
    if !matches!(input.get(*pos), Some(Token::BraceL)) {
        return Err(format!("unexpected token: {:?}", input.get(*pos)));
    }
    *pos += 1;
    let mut statements = Vec::new();
    loop {
        match input.get(*pos) {
            Some(Token::BraceR) => {
                *pos += 1;
                return Ok(Block { statements });
            }
            None => return Err(String::from("Unexpected EOF. '}' is expected.")),
            _ => statements.push(parse_stmt(input, pos)?),
        }
    }
}

fn expect_paren(input: &[Token], pos: &mut usize, left: bool) -> Result<(), String> {
    let matches = if left {
        matches!(input.get(*pos), Some(Token::ParL))
    } else {
        matches!(input.get(*pos), Some(Token::ParR))
    };
    if !matches {
        return Err(format!("unexpected token: {:?}", input.get(*pos)));
    }
    *pos += 1;
    Ok(())
}

fn parse_stmt(input: &[Token], pos: &mut usize) -> Result<Stmt, String> {
    match input.get(*pos) {
        Some(Token::BraceL) => Ok(Stmt::Block(parse_block(input, pos)?)),
        Some(Token::If) => {
            *pos += 1;
            expect_paren(input, pos, true)?;
            let condition = parse_expr(input, pos)?;
            expect_paren(input, pos, false)?;
            let then_branch = Box::new(parse_stmt(input, pos)?);
            let else_branch = if matches!(input.get(*pos), Some(Token::Else)) {
                *pos += 1;
                Some(Box::new(parse_stmt(input, pos)?))
            } else {
                None
            };
            Ok(Stmt::If {
                condition,
                then_branch,
                else_branch,
            })
        }
        Some(Token::While) => {
            *pos += 1;
            expect_paren(input, pos, true)?;
            let condition = parse_expr(input, pos)?;
            expect_paren(input, pos, false)?;
            Ok(Stmt::While {
                condition,
                body: Box::new(parse_stmt(input, pos)?),
            })
        }
        Some(Token::For) => {
            *pos += 1;
            expect_paren(input, pos, true)?;
            let init = parse_optional_expr(input, pos, TokenKind::Semi)?;
            let condition = parse_optional_expr(input, pos, TokenKind::Semi)?;
            let update = parse_optional_expr(input, pos, TokenKind::ParR)?;
            Ok(Stmt::For {
                init,
                condition,
                update,
                body: Box::new(parse_stmt(input, pos)?),
            })
        }
        Some(Token::Return) => {
            *pos += 1;
            let expr = parse_expr(input, pos)?;
            expect_semicolon(input, pos)?;
            Ok(Stmt::Return(expr))
        }
        _ => {
            let expr = parse_expr(input, pos)?;
            expect_semicolon(input, pos)?;
            Ok(Stmt::Expr(expr))
        }
    }
}

#[derive(Clone, Copy)]
enum TokenKind {
    Semi,
    ParR,
}

fn is_delimiter(token: Option<&Token>, kind: TokenKind) -> bool {
    match kind {
        TokenKind::Semi => matches!(token, Some(Token::Semi)),
        TokenKind::ParR => matches!(token, Some(Token::ParR)),
    }
}

fn parse_optional_expr(
    input: &[Token],
    pos: &mut usize,
    delimiter: TokenKind,
) -> Result<Option<Expr>, String> {
    if is_delimiter(input.get(*pos), delimiter) {
        *pos += 1;
        return Ok(None);
    }
    let expr = parse_expr(input, pos)?;
    if !is_delimiter(input.get(*pos), delimiter) {
        return Err(format!("unexpected token: {:?}", input.get(*pos)));
    }
    *pos += 1;
    Ok(Some(expr))
}

fn expect_semicolon(input: &[Token], pos: &mut usize) -> Result<(), String> {
    if !matches!(input.get(*pos), Some(Token::Semi)) {
        return Err(String::from("';' is required at the end of statements."));
    }
    *pos += 1;
    Ok(())
}

fn parse_function(input: &[Token], pos: &mut usize) -> Result<Function, String> {
    let name = match input.get(*pos) {
        Some(Token::Id(name)) => name.clone(),
        token => return Err(format!("unexpected token: {token:?}")),
    };
    *pos += 1;
    expect_paren(input, pos, true)?;

    let mut params = Vec::new();
    if matches!(input.get(*pos), Some(Token::ParR)) {
        *pos += 1;
    } else {
        loop {
            match input.get(*pos) {
                Some(Token::Id(param)) => params.push(param.clone()),
                token => return Err(format!("unexpected token: {token:?}")),
            }
            *pos += 1;
            match input.get(*pos) {
                Some(Token::Comma) => *pos += 1,
                Some(Token::ParR) => {
                    *pos += 1;
                    break;
                }
                token => return Err(format!("unexpected token: {token:?}")),
            }
        }
    }

    Ok(Function {
        name,
        params,
        body: parse_block(input, pos)?,
    })
}

pub fn parse(input: &[Token]) -> Result<Program, String> {
    let mut pos = 0;
    let mut functions = Vec::new();
    while pos < input.len() {
        functions.push(parse_function(input, &mut pos)?);
    }
    Ok(Program { functions })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_source(source: &str) -> Result<Program, String> {
        parse(&crate::lexer::lex(source).unwrap())
    }

    #[test]
    fn parses_functions_parameters_and_calls() {
        let program =
            parse_source("add(a, b) { return a + b; } main() { return 1 + add(2, add(3, 4)); }")
                .unwrap();

        assert_eq!(program.functions.len(), 2);
        assert_eq!(program.functions[0].name, "add");
        assert_eq!(program.functions[0].params, ["a", "b"]);
        assert!(matches!(
            program.functions[1].body.statements[0],
            Stmt::Return(Expr::Binary { .. })
        ));
    }

    #[test]
    fn parses_control_flow_and_blocks() {
        let program = parse_source(
            "main() { if (1) { a = 1; } else a = 2; while (a) a = a - 1; for (i = 0; i < 3; i = i + 1) {} }",
        )
        .unwrap();

        assert_eq!(program.functions[0].body.statements.len(), 3);
        assert!(matches!(
            program.functions[0].body.statements[2],
            Stmt::For { .. }
        ));
    }

    #[test]
    fn assignment_is_right_associative() {
        let program = parse_source("main() { a = b = 1; }").unwrap();
        let Stmt::Expr(Expr::Assign { name, value }) = &program.functions[0].body.statements[0]
        else {
            panic!("expected assignment");
        };
        assert_eq!(name, "a");
        assert!(matches!(value.as_ref(), Expr::Assign { name, .. } if name == "b"));
    }

    #[test]
    fn rejects_invalid_assignment_target_and_trailing_commas() {
        assert!(parse_source("main() { (1 + 2) = 3; }").is_err());
        assert!(parse_source("main(a,) { return a; }").is_err());
        assert!(parse_source("main() { return f(1,); }").is_err());
    }

    #[test]
    fn rejects_unclosed_block() {
        assert_eq!(
            parse_source("main() { return 1;"),
            Err(String::from("Unexpected EOF. '}' is expected."))
        );
    }
}
