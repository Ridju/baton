use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum Token {
    Return,
    IntKeyword,
    BoolKeyword,
    FloatKeyword,
    StringKeyword,
    Identifier(String),
    IntNumber(String),
    Bool(bool),
    FloatNumber(String),
    String(String),
    StructKeyword,

    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
    DoubleEqual,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Dot,

    If,
    Else,
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
                '+' => {
                    self.chars.next();
                    tokens.push(Token::Plus);
                    self.column += 1;
                }
                '-' => {
                    self.chars.next();
                    tokens.push(Token::Minus);
                    self.column += 1;
                }
                '*' => {
                    self.chars.next();
                    tokens.push(Token::Star);
                    self.column += 1;
                }
                '/' => {
                    self.chars.next();
                    tokens.push(Token::Slash);
                    self.column += 1;
                }
                '=' => {
                    self.chars.next();
                    self.column += 1;
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        tokens.push(Token::DoubleEqual);
                        self.column += 1;
                    } else {
                        tokens.push(Token::Equal);
                    }
                }
                '<' => {
                    self.chars.next();
                    self.column += 1;
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        tokens.push(Token::LessOrEqual);
                        self.column += 1;
                    } else {
                        tokens.push(Token::LessThan);
                    }
                }
                '>' => {
                    self.chars.next();
                    self.column += 1;
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        tokens.push(Token::GreaterOrEqual);
                        self.column += 1;
                    } else {
                        tokens.push(Token::GreaterThan);
                    }
                }
                ',' => {
                    self.chars.next();
                    self.column += 1;
                    tokens.push(Token::Comma);
                }
                '"' => {
                    tokens.push(self.scan_string()?);
                }
                '.' => {
                    self.chars.next();
                    self.column += 1;
                    tokens.push(Token::Dot);
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

    fn scan_string(&mut self) -> Result<Token, ScannerError> {
        self.chars.next();
        self.column += 1;

        let mut buffer = String::new();
        while let Some(&c) = self.chars.peek() {
            match c {
                '"' => {
                    self.chars.next();
                    self.column += 1;
                    return Ok(Token::String(buffer));
                }
                '\n' => {
                    self.chars.next();
                    self.line += 1;
                    self.column = 1;
                    buffer.push('\n');
                }
                other => {
                    self.chars.next();
                    buffer.push(other);
                    self.column += 1;
                }
            }
        }

        Err(ScannerError {
            message: "Unterminated string literal".to_string(),
            line: self.line,
            column: self.column,
        })
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
            "bool" => Token::BoolKeyword,
            "if" => Token::If,
            "else" => Token::Else,
            "false" => Token::Bool(false),
            "true" => Token::Bool(true),
            "float" => Token::FloatKeyword,
            "string" => Token::StringKeyword,
            "struct" => Token::StructKeyword,
            _ => Token::Identifier(token_string),
        };

        Ok(token)
    }

    fn scan_number(&mut self) -> Result<Token, ScannerError> {
        let mut token_string = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_numeric() || c == '.' {
                token_string.push(c);
                self.chars.next();
                self.column += 1;
            } else {
                break;
            }
        }
        if token_string.contains(".") {
            Ok(Token::FloatNumber(token_string))
        } else {
            Ok(Token::IntNumber(token_string))
        }
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

    #[test]
    fn test_plus_token() {
        let input = "40 + 2";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();
        assert_eq!(
            result,
            vec![
                Token::IntNumber("40".to_string()),
                Token::Plus,
                Token::IntNumber("2".to_string())
            ]
        );
    }

    #[test]
    fn test_minus_token() {
        let input = "40 - 2";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();
        assert_eq!(
            result,
            vec![
                Token::IntNumber("40".to_string()),
                Token::Minus,
                Token::IntNumber("2".to_string())
            ]
        );
    }

    #[test]
    fn test_star_token() {
        let input = "40 * 2";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();
        assert_eq!(
            result,
            vec![
                Token::IntNumber("40".to_string()),
                Token::Star,
                Token::IntNumber("2".to_string())
            ]
        );
    }

    #[test]
    fn test_slash_token() {
        let input = "40 / 2";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();
        assert_eq!(
            result,
            vec![
                Token::IntNumber("40".to_string()),
                Token::Slash,
                Token::IntNumber("2".to_string())
            ]
        );
    }

    #[test]
    fn test_variable() {
        let input = "int i = 0;";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();
        assert_eq!(
            result,
            vec![
                Token::IntKeyword,
                Token::Identifier("i".to_string()),
                Token::Equal,
                Token::IntNumber("0".to_string()),
                Token::Semicolon
            ]
        );
    }

    #[test]
    fn test_if_else() {
        let input = "if(a==1){}else{}";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();
        assert_eq!(
            result,
            vec![
                Token::If,
                Token::LeftParen,
                Token::Identifier("a".to_string()),
                Token::DoubleEqual,
                Token::IntNumber("1".to_string()),
                Token::RightParen,
                Token::LeftBrace,
                Token::RightBrace,
                Token::Else,
                Token::LeftBrace,
                Token::RightBrace
            ]
        );
    }

    #[test]
    fn test_greater_less_equals() {
        let input = "<= >= == < >";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();

        assert_eq!(
            result,
            vec![
                Token::LessOrEqual,
                Token::GreaterOrEqual,
                Token::DoubleEqual,
                Token::LessThan,
                Token::GreaterThan
            ]
        )
    }

    #[test]
    fn test_parameters() {
        let input = "int main(int a, int b)";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();

        assert_eq!(
            result,
            vec![
                Token::IntKeyword,
                Token::Identifier("main".to_string()),
                Token::LeftParen,
                Token::IntKeyword,
                Token::Identifier("a".to_string()),
                Token::Comma,
                Token::IntKeyword,
                Token::Identifier("b".to_string()),
                Token::RightParen,
            ]
        );
    }

    #[test]
    fn test_bool() {
        let input = "bool a = false;";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();

        assert_eq!(
            result,
            vec![
                Token::BoolKeyword,
                Token::Identifier("a".to_string()),
                Token::Equal,
                Token::Bool(false),
                Token::Semicolon
            ]
        );
    }

    #[test]
    fn test_string() {
        let input = "string a = \"test\"";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();

        assert_eq!(
            result,
            vec![
                Token::StringKeyword,
                Token::Identifier("a".to_string()),
                Token::Equal,
                Token::String("test".to_string())
            ]
        );
    }

    #[test]
    fn test_float() {
        let input = "float a = 3.0";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();

        assert_eq!(
            result,
            vec![
                Token::FloatKeyword,
                Token::Identifier("a".to_string()),
                Token::Equal,
                Token::FloatNumber("3.0".to_string()),
            ]
        );
    }
#[test]
    fn test_struct_and_member_access() {
        let input = "struct Point { int x; int y; } Point p; p.x = 5;";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();
        assert_eq!(
            result,
            vec![
                Token::StructKeyword,
                Token::Identifier("Point".to_string()),
                Token::LeftBrace,
                Token::IntKeyword,
                Token::Identifier("x".to_string()),
                Token::Semicolon,
                Token::IntKeyword,
                Token::Identifier("y".to_string()),
                Token::Semicolon,
                Token::RightBrace,
                Token::Identifier("Point".to_string()),
                Token::Identifier("p".to_string()),
                Token::Semicolon,
                Token::Identifier("p".to_string()),
                Token::Dot,
                Token::Identifier("x".to_string()),
                Token::Equal,
                Token::IntNumber("5".to_string()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn test_boolean_true_and_comparisons() {
        let input = "bool flag = true; if (flag == true) {}";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();
        assert_eq!(
            result,
            vec![
                Token::BoolKeyword,
                Token::Identifier("flag".to_string()),
                Token::Equal,
                Token::Bool(true),
                Token::Semicolon,
                Token::If,
                Token::LeftParen,
                Token::Identifier("flag".to_string()),
                Token::DoubleEqual,
                Token::Bool(true),
                Token::RightParen,
                Token::LeftBrace,
                Token::RightBrace,
            ]
        );
    }

    #[test]
    fn test_unterminated_string_error() {
        let input = "string s = \"hello;";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source();
        assert_eq!(
            result,
            Err(ScannerError {
                message: "Unterminated string literal".to_string(),
                line: 1,
                column: 19,
            })
        );
    }

    #[test]
    fn test_multiline_string() {
        let input = "\"line1\nline2\"";
        let mut sc = Scanner::new(input);
        let result = sc.scan_source().unwrap();
        assert_eq!(
            result,
            vec![Token::String("line1\nline2".to_string())]
        );
    }
}
