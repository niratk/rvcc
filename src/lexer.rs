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
    let mut tokens = Vec::new();
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
                tokens.push(Token::Num(num));
            }
            c if c.is_ascii_whitespace() => pos += 1,
            b'+' => {
                tokens.push(Token::Plus);
                pos += 1;
            }
            b'-' => {
                tokens.push(Token::Minus);
                pos += 1;
            }
            b'*' => {
                tokens.push(Token::Mult);
                pos += 1;
            }
            b'/' => {
                tokens.push(Token::Div);
                pos += 1;
            }
            b'(' => {
                tokens.push(Token::ParL);
                pos += 1;
            }
            b')' => {
                tokens.push(Token::ParR);
                pos += 1;
            }
            b'>' => match input.get(pos + 1) {
                Some(b'=') => {
                    tokens.push(Token::Geq);
                    pos += 2;
                }
                _ => {
                    tokens.push(Token::Gt);
                    pos += 1;
                }
            },
            b'<' => match input.get(pos + 1) {
                Some(b'=') => {
                    tokens.push(Token::Leq);
                    pos += 2;
                }
                _ => {
                    tokens.push(Token::Lt);
                    pos += 1;
                }
            },
            b'=' => match input.get(pos + 1) {
                Some(b'=') => {
                    tokens.push(Token::Eq);
                    pos += 2;
                }
                _ => return invalid_character(pos),
            },
            b'!' => match input.get(pos + 1) {
                Some(b'=') => {
                    tokens.push(Token::Neq);
                    pos += 2;
                }
                _ => return invalid_character(pos),
            },
            _ => return invalid_character(pos),
        }
    }

    Ok(tokens)
}

fn invalid_character<T>(pos: usize) -> Result<T, String> {
    eprintln!("Invalid Character at {}", pos);
    Err(format!("Invalid Character at {}", pos))
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
