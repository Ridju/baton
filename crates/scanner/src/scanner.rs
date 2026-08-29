use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    line: usize,
    column: usize,
    kind: TokenKind<'a>,
}

impl<'a> Token<'a> {
    fn new(line: usize, column: usize, kind: TokenKind<'a>) -> Self {
        Self { line, column, kind }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {
    Return,
    IntKeyword,
    BoolKeyword,
    FloatKeyword,
    StringKeyword,
    PrintKeyword,
    Identifier(&'a str),
    IntNumber(&'a str),
    Bool(bool),
    FloatNumber(&'a str),
    String(&'a str),
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
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Dot,

    If,
    Else,
    While,
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
                    self.column += 1;
                    if self.chars.peek() == Some(&'/') {
                        self.chars.next();
                        self.column += 1;
                        while let Some(&next_c) = self.chars.peek() {
                            if next_c == '\n' {
                                break;
                            }
                            self.chars.next();
                            self.column += 1;
                        }
                    } else {
                        tokens.push(Token::Slash);
                    }
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
                '[' => {
                    self.chars.next();
                    self.column += 1;
                    tokens.push(Token::LeftBracket);
                }
                ']' => {
                    self.chars.next();
                    self.column += 1;
                    tokens.push(Token::RightBracket);
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
            "while" => Token::While,
            "print" => Token::PrintKeyword,
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
