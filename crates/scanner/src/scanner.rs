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
    source: &'a [u8],
    cursor: usize,
    line: usize,
    column: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            cursor: 0,
            line: 1,
            column: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.source.len()
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source[self.cursor]
        }
    }

    fn peek_next(&self) -> u8 {
        if self.cursor + 1 >= self.source.len() {
            b'\0'
        } else {
            self.source[self.cursor + 1]
        }
    }

    fn advance(&mut self) -> u8 {
        let byte = self.source[self.cursor];
        self.cursor += 1;
        self.column += 1;
        byte
    }

    fn create_token(&self, kind: TokenKind<'a>) -> Token {
        Token {
            line: self.line,
            column: self.column,
            kind,
        }
    }

    pub fn scan_source(&mut self) -> Result<Vec<Token>, ScannerError> {
        let mut tokens: Vec<Token> = Vec::new();

        while !self.is_at_end() {
            let start = self.cursor;
            let c = self.advance();

            match c {
                b'\n' => {
                    self.line += 1;
                    self.column += 1;
                }
                c if c.is_ascii_whitespace() => {}
                b'/' => {
                    if self.peek() == b'/' {
                        self.advance();
                        while self.peek() != b'\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        tokens.push(self.create_token(TokenKind::Slash));
                    }
                }
                b'=' => {
                    if self.peek() == b'=' {
                        self.advance();
                        tokens.push(self.create_token(TokenKind::DoubleEqual));
                    } else {
                        tokens.push(self.create_token(TokenKind::Equal));
                    }
                }
                b'<' => {
                    if self.peek() == b'=' {
                        self.advance();
                        tokens.push(self.create_token(TokenKind::LessOrEqual));
                    } else {
                        tokens.push(self.create_token(TokenKind::LessThan));
                    }
                }
                b'>' => {
                    if self.peek() == b'=' {
                        self.advance();
                        tokens.push(self.create_token(TokenKind::GreaterOrEqual));
                    } else {
                        tokens.push(self.create_token(TokenKind::GreaterOrEqual));
                    }
                }
                b'(' => tokens.push(self.create_token(TokenKind::LeftParen)),
                b')' => tokens.push(self.create_token(TokenKind::RightParen)),
                b'{' => tokens.push(self.create_token(TokenKind::LeftBrace)),
                b'}' => tokens.push(self.create_token(TokenKind::RightBrace)),
                b'[' => tokens.push(self.create_token(TokenKind::LeftBracket)),
                b']' => tokens.push(self.create_token(TokenKind::RightBracket)),
                b';' => tokens.push(self.create_token(TokenKind::Semicolon)),
                b',' => tokens.push(self.create_token(TokenKind::Comma)),
                b'.' => tokens.push(self.create_token(TokenKind::Dot)),
                b'+' => tokens.push(self.create_token(TokenKind::Plus)),
                b'-' => tokens.push(self.create_token(TokenKind::Minus)),
                b'*' => tokens.push(self.create_token(TokenKind::Star)),

                b'"' => tokens.push(self.scan_string()?),
                c if c.is_ascii_digit() => tokens.push(self.scan_number()?),
                c if c.is_ascii_alphabetic() || c == b'_' => {
                    tokens.push(self.identifier_or_keyword()?)
                }
                other => {
                    return Err(ScannerError {
                        message: format!("Unexpected characgter: {}", other as char),
                        line: self.line,
                        column: self.column,
                    });
                }
            }
        }

        Ok(tokens)
    }

    fn scan_string(&mut self) -> Result<Token, ScannerError> {
        let start = self.cursor;
        let start_column = self.column.saturating_sub(1);

        while !self.is_at_end() {
            let c = self.advance();
            match c {
                b'"' => {
                    let bytes = &self.source[start..self.cursor - 1];
                    let text = std::str::from_utf8(bytes).map_err(|_| ScannerError {
                        message: "Invalid UTF-8 sequence in source".to_string(),
                        line: self.line,
                        column: start_column,
                    })?;
                    let token = Token::new(self.line, self.column, TokenKind::String(text));
                    return Ok(token);
                }
                b'\n' => {
                    self.line += 1;
                    self.column = 1;
                }
                _ => {}
            }
        }

        Err(ScannerError {
            message: "Unterminated string literal".to_string(),
            line: self.line,
            column: self.column,
        })
    }

    fn identifier_or_keyword(&mut self) -> Result<Token, ScannerError> {
        let start = self.cursor - 1;
        let start_column = self.column.saturating_sub(1);

        while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
            self.advance();
        }

        let bytes = &self.source[start..self.cursor];
        let text = std::str::from_utf8(bytes).map_err(|_| ScannerError {
            message: "Invalid UTF-8 sequence in source".to_string(),
            line: self.line,
            column: start_column,
        })?;

        let kind = match text {
            "return" => TokenKind::Return,
            "int" => TokenKind::IntKeyword,
            "bool" => TokenKind::BoolKeyword,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "print" => TokenKind::PrintKeyword,
            "false" => TokenKind::Bool(false),
            "true" => TokenKind::Bool(true),
            "float" => TokenKind::FloatKeyword,
            "string" => TokenKind::StringKeyword,
            "struct" => TokenKind::StructKeyword,
            _ => TokenKind::Identifier(text),
        };

        Ok(Token::new(self.line, start_column, kind))
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
