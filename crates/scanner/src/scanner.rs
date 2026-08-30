#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub line: usize,
    pub column: usize,
    pub kind: TokenKind<'a>,
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
    EoF,
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

    pub fn scan_source(&mut self) -> Result<Vec<Token<'a>>, ScannerError> {
        let mut tokens: Vec<Token<'a>> = Vec::new();

        while !self.is_at_end() {
            let c = self.advance();

            match c {
                b'\n' => {
                    self.line += 1;
                    self.column = 1;
                }
                c if c.is_ascii_whitespace() => {}
                b'/' => {
                    if self.peek() == b'/' {
                        self.advance();
                        while self.peek() != b'\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        tokens.push(Token::new(self.line, self.column - 1, TokenKind::Slash));
                    }
                }
                b'=' => {
                    if self.peek() == b'=' {
                        self.advance();
                        tokens.push(Token::new(
                            self.line,
                            self.column - 2,
                            TokenKind::DoubleEqual,
                        ));
                    } else {
                        tokens.push(Token::new(self.line, self.column - 1, TokenKind::Equal));
                    }
                }
                b'<' => {
                    if self.peek() == b'=' {
                        self.advance();
                        tokens.push(Token::new(
                            self.line,
                            self.column - 2,
                            TokenKind::LessOrEqual,
                        ));
                    } else {
                        tokens.push(Token::new(self.line, self.column - 1, TokenKind::LessThan));
                    }
                }
                b'>' => {
                    if self.peek() == b'=' {
                        self.advance();
                        tokens.push(Token::new(
                            self.line,
                            self.column - 2,
                            TokenKind::GreaterOrEqual,
                        ));
                    } else {
                        tokens.push(Token::new(
                            self.line,
                            self.column - 1,
                            TokenKind::GreaterThan,
                        ));
                    }
                }
                b'(' => tokens.push(Token::new(self.line, self.column - 1, TokenKind::LeftParen)),
                b')' => tokens.push(Token::new(
                    self.line,
                    self.column - 1,
                    TokenKind::RightParen,
                )),
                b'{' => tokens.push(Token::new(self.line, self.column - 1, TokenKind::LeftBrace)),
                b'}' => tokens.push(Token::new(
                    self.line,
                    self.column - 1,
                    TokenKind::RightBrace,
                )),
                b'[' => tokens.push(Token::new(
                    self.line,
                    self.column - 1,
                    TokenKind::LeftBracket,
                )),
                b']' => tokens.push(Token::new(
                    self.line,
                    self.column - 1,
                    TokenKind::RightBracket,
                )),
                b';' => tokens.push(Token::new(self.line, self.column - 1, TokenKind::Semicolon)),
                b',' => tokens.push(Token::new(self.line, self.column - 1, TokenKind::Comma)),
                b'.' => tokens.push(Token::new(self.line, self.column - 1, TokenKind::Dot)),
                b'+' => tokens.push(Token::new(self.line, self.column - 1, TokenKind::Plus)),
                b'-' => tokens.push(Token::new(self.line, self.column - 1, TokenKind::Minus)),
                b'*' => tokens.push(Token::new(self.line, self.column - 1, TokenKind::Star)),

                b'"' => {
                    let string_token = self.scan_string()?;
                    tokens.push(string_token);
                }
                c if c.is_ascii_digit() => {
                    let num_token = self.scan_number()?;
                    tokens.push(num_token);
                }
                c if c.is_ascii_alphabetic() || c == b'_' => {
                    let identifier = self.identifier_or_keyword()?;
                    tokens.push(identifier);
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
        tokens.push(Token::new(self.line, self.column, TokenKind::EoF));
        Ok(tokens)
    }

    fn scan_string(&mut self) -> Result<Token<'a>, ScannerError> {
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
                    let token = Token::new(self.line, start_column, TokenKind::String(text));
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

    fn identifier_or_keyword(&mut self) -> Result<Token<'a>, ScannerError> {
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

    fn scan_number(&mut self) -> Result<Token<'a>, ScannerError> {
        let start = self.cursor - 1;
        let start_column = self.column.saturating_sub(1);
        let mut is_float = false;

        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let bytes = &self.source[start..self.cursor];
        let text = std::str::from_utf8(bytes).map_err(|_| ScannerError {
            message: "Invalid UTF-8 sequence in source".to_string(),
            line: self.line,
            column: start_column,
        })?;

        let kind = if is_float {
            TokenKind::FloatNumber(text)
        } else {
            TokenKind::IntNumber(text)
        };

        Ok(Token::new(self.line, self.column, kind))
    }
}
