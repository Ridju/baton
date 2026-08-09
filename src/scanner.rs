use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum Token {
    Return,
    IntKeyword,
    Identifier(String),
    IntNumber(String),

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Semicolon,
}

#[derive(Debug, PartialEq)]
pub struct ScannerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            chars: source.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    pub fn scan_source(&mut self) -> Result<Vec<Token>, ScannerError> {
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(&c) = self.chars.peek() {
            match c {
                '\n' => {
                    self.chars.next();
                    self.line += 1;
                    self.column = 1;
                }
                c if c.is_whitespace() => {
                    self.chars.next();
                    self.column += 1;
                }
                c if c.is_alphabetic() => {
                    tokens.push(self.identifier_or_keyword()?);
                }
                c if c.is_numeric() => {
                    tokens.push(self.scan_number()?);
                }
                '(' => {
                    self.chars.next();
                    tokens.push(Token::LeftParen);
                    self.column += 1;
                }
                ')' => {
                    self.chars.next();
                    tokens.push(Token::RightParen);
                    self.column += 1;
                }
                '{' => {
                    self.chars.next();
                    tokens.push(Token::LeftBrace);
                    self.column += 1;
                }
                '}' => {
                    self.chars.next();
                    tokens.push(Token::RightBrace);
                    self.column += 1;
                }
                ';' => {
                    self.chars.next();
                    tokens.push(Token::Semicolon);
                    self.column += 1;
                }
                other => {
                    return Err(ScannerError {
                        message: format!("Unexpected character: '{}'", other),
                        line: self.line,
                        column: self.column,
                    });
                }
            }
        }
        Ok(tokens)
    }

    fn identifier_or_keyword(&mut self) -> Result<Token, ScannerError> {
        let mut token_string = String::new();

        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                token_string.push(c);
                self.chars.next();
                self.column += 1;
            } else {
                break;
            }
        }

        let token = match token_string.as_str() {
            "return" => Token::Return,
            "int" => Token::IntKeyword,
            _ => Token::Identifier(token_string),
        };

        Ok(token)
    }

    fn scan_number(&mut self) -> Result<Token, ScannerError> {
        let mut token_string = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_numeric() {
                token_string.push(c);
                self.chars.next();
                self.column += 1;
            } else {
                break;
            }
        }

        Ok(Token::IntNumber(token_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_main_programm() {
        let input = r#"
        int main() {
            return 42;
        } 
        "#;

        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::IntKeyword,
                Token::Identifier("main".to_string()),
                Token::LeftParen,
                Token::RightParen,
                Token::LeftBrace,
                Token::Return,
                Token::IntNumber("42".to_string()),
                Token::Semicolon,
                Token::RightBrace
            ]
        );
    }

    #[test]
    fn test_return_token() {
        let input = "return";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(tokens, vec![Token::Return]);
    }

    #[test]
    fn test_int_token() {
        let input = "int";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(tokens, vec![Token::IntKeyword]);
    }

    #[test]
    fn test_identifier() {
        let input = "my_variable123";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(
            tokens,
            vec![Token::Identifier("my_variable123".to_string())]
        );
    }

    #[test]
    fn test_keyword_in_identifier() {
        let input = "my_return_int_var";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(
            tokens,
            vec![Token::Identifier("my_return_int_var".to_string())]
        );
    }

    #[test]
    fn test_left_paren_token() {
        let input = "(";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(tokens, vec![Token::LeftParen]);
    }

    #[test]
    fn test_right_paren_token() {
        let input = ")";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(tokens, vec![Token::RightParen]);
    }

    #[test]
    fn test_left_brace_token() {
        let input = "{";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(tokens, vec![Token::LeftBrace]);
    }

    #[test]
    fn test_right_brace_token() {
        let input = "}";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(tokens, vec![Token::RightBrace]);
    }

    #[test]
    fn test_semicolon_token() {
        let input = ";";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(tokens, vec![Token::Semicolon]);
    }

    #[test]
    fn test_int_number() {
        let input = "42";
        let mut sc = Scanner::new(input);
        let tokens = sc.scan_source().unwrap();
        assert_eq!(tokens, vec![Token::IntNumber("42".to_string())]);
    }

    #[test]
    fn test_unexpected_character_at_start() {
        let input = "@int main() {}";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source();

        assert_eq!(
            result,
            Err(ScannerError {
                message: "Unexpected character: '@'".to_string(),
                line: 1,
                column: 1,
            })
        );
    }

    #[test]
    fn test_unexpected_character_multiline() {
        let input = "int main() {\n    return 42#;\n}";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source();

        assert_eq!(
            result,
            Err(ScannerError {
                message: "Unexpected character: '#'".to_string(),
                line: 2,
                column: 14,
            })
        );
    }

    #[test]
    fn test_unexpected_symbol_in_middle() {
        let input = "int $variable;";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source();

        assert_eq!(
            result,
            Err(ScannerError {
                message: "Unexpected character: '$'".to_string(),
                line: 1,
                column: 5,
            })
        );
    }

    #[test]
    fn test_unexpected_question_mark() {
        let input = "int main(?) {}";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source();

        assert_eq!(
            result,
            Err(ScannerError {
                message: "Unexpected character: '?'".to_string(),
                line: 1,
                column: 10,
            })
        );
    }
}
