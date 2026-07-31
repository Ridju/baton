use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum Token {
    Return(String),
    IntKeyword(String),
    Identifier(String),

    IntNumber(String),
}

#[derive(Debug)]
struct ScannerError;

pub fn scan_source(source: &str) -> Vec<Token> {
    let mut chars = source.chars().peekable();
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(&c) = chars.peek() {
        if c.is_alphabetic() {
            let token = identifier_or_keyword(&mut chars);
            tokens.push(token);
        }
    }

    tokens
}

fn identifier_or_keyword(chars: &mut Peekable<Chars<'_>>) -> Token {
    let mut token_string = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            token_string.push(c);
            chars.next();
        } else {
            break;
        }
    }

    match token_string.as_str() {
        "return" => {
            return Token::Return(token_string);
        }
        "int" => {
            return Token::IntKeyword(token_string);
        }
        _ => {
            return Token::Identifier(token_string);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_token() {
        let input = "return";
        let tokens = scan_source(input);
        assert_eq!(tokens, vec![Token::Return("return".to_string())]);
    }

    #[test]
    fn test_int_token() {
        let input = "int";
        let tokens = scan_source(input);
        assert_eq!(tokens, vec![Token::IntKeyword("int".to_string())]);
    }

    #[test]
    fn test_identifier() {
        let input = "my_variable123";
        let tokens = scan_source(input);
        assert_eq!(
            tokens,
            vec![Token::Identifier("my_variable123".to_string())]
        );
    }
}
