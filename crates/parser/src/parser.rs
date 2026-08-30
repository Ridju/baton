use scanner::Token;
use scanner::TokenKind;

#[derive(Debug, PartialEq)]
pub enum ParserError {
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },
    UnknownType(String),
    NotImplemented(String),
    InvalidIntLiteral {
        raw: String,
        line: usize,
        column: usize,
    },
    InvalidFloatLiteral {
        raw: String,
        line: usize,
        column: usize,
    },
    UnexpectedEoF,
}

#[derive(Debug, PartialEq)]
pub enum Type<'a> {
    Int,
    Bool,
    Float,
    String,
    Struct(&'a str),
    Array(Box<Type<'a>>),
}

#[derive(Debug, PartialEq)]
pub struct Parameter<'a> {
    pub name: &'a str,
    pub param_typ: Type<'a>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionDeclData<'a> {
    pub name: &'a str,
    pub return_type: Type<'a>,
    pub parameter: Vec<Parameter<'a>>,
    pub body: Box<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    DoubleEqual,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
}

#[derive(Debug, PartialEq)]
pub struct BinaryExpData<'a> {
    pub left: Box<AstNode<'a>>,
    pub right: Box<AstNode<'a>>,
    pub operator: BinaryOperator,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct VarDeclData<'a> {
    pub name: &'a str,
    pub var_typ: Type<'a>,
    pub initializer: Option<Box<AstNode<'a>>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct AssignmentData<'a> {
    pub target: Box<AstNode<'a>>,
    pub value: Box<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct IfElseData<'a> {
    pub condition_expr: Box<AstNode<'a>>,
    pub if_branch: Box<AstNode<'a>>,
    pub else_branch: Option<Box<AstNode<'a>>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct CallData<'a> {
    pub name: &'a str,
    pub arguments: Vec<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct MemberAccessData<'a> {
    pub object: Box<AstNode<'a>>,
    pub member: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct ArrayIndexData<'a> {
    pub array: Box<AstNode<'a>>,
    pub index: Box<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct WhileLoopData<'a> {
    pub condition_expr: Box<AstNode<'a>>,
    pub body: Box<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct StructDeclData<'a> {
    pub name: &'a str,
    pub fields: Vec<Parameter<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct BlockStatementData<'a> {
    pub statements: Vec<AstNode<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub struct ReturnData<'a> {
    pub value: Option<Box<AstNode<'a>>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub enum AstNode<'a> {
    Programm(Vec<AstNode<'a>>),
    FunctionDecl(FunctionDeclData<'a>),
    ReturnStatement(ReturnData<'a>),
    IntLiteralExpr(i16),
    BoolLiteralExpr(bool),
    BlockStatement(BlockStatementData<'a>),
    BinaryExpr(BinaryExpData<'a>),
    IfElseStatement(IfElseData<'a>),
    VariableExpr(&'a str),
    VarDeclStatement(VarDeclData<'a>),
    AssignmentStatement(AssignmentData<'a>),
    CallExpr(CallData<'a>),
    FloatLiteralExpr(f64),
    StringLiteralExpr(&'a str),
    StructDecl(StructDeclData<'a>),
    MemberAccessExpr(MemberAccessData<'a>),
    ArrayIndexExpr(ArrayIndexData<'a>),
    WhileStatement(WhileLoopData<'a>),
    PrintStatement(Box<AstNode<'a>>),
}

pub struct Parser<'a> {
    tokens: &'a [Token<'a>],
    cursor: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Parser { tokens, cursor: 0 }
    }

    fn is_at_end(&self) -> bool {
        match self.tokens.get(self.cursor) {
            Some(token) => token.kind == TokenKind::EoF,
            None => true,
        }
    }

    fn peek(&self) -> Option<&'a Token<'a>> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) -> Option<&'a Token<'a>> {
        let token = self.tokens.get(self.cursor);
        self.cursor += 1;
        token
    }

    fn expect(&mut self, expected: TokenKind) -> Result<&'a Token<'a>, ParserError> {
        match self.peek() {
            Some(token) if token.kind == expected => {
                self.advance().ok_or(ParserError::UnexpectedEoF)
            }
            Some(token) => Err(ParserError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", token.kind),
                line: token.line,
                column: token.column,
            }),
            None => Err(ParserError::UnexpectedEoF),
        }
    }

    fn check(&self, expected: TokenKind) -> bool {
        if let Some(token) = self.peek() {
            token.kind == expected
        } else {
            false
        }
    }

    pub fn parse(&mut self) -> Result<AstNode<'a>, ParserError> {
        let mut nodes = Vec::new();

        while let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::StructKeyword => {
                    nodes.push(self.parse_struct_decl()?);
                }
                TokenKind::IntKeyword
                | TokenKind::BoolKeyword
                | TokenKind::FloatKeyword
                | TokenKind::StringKeyword
                | TokenKind::Identifier(_) => {
                    let is_function = match self.tokens.get(self.cursor + 2) {
                        Some(t) => t.kind == TokenKind::LeftParen,
                        None => false,
                    };

                    if is_function {
                        nodes.push(self.parse_function()?);
                    } else {
                        return Err(ParserError::NotImplemented(
                            "Global variable parsing not implemented".to_string(),
                        ));
                    }
                }
                TokenKind::EoF => break,
                other => {
                    return Err(ParserError::UnexpectedToken {
                        expected: "type keyword (int, bool, float, string) or struct".to_string(),
                        found: format!("{:?}", other),
                        line: token.line,
                        column: token.column,
                    });
                }
            }
        }

        Ok(AstNode::Programm(nodes))
    }

    fn parse_struct_decl(&mut self) -> Result<AstNode<'a>, ParserError> {
        self.expect(TokenKind::StructKeyword)?;

        let name_token = self.advance().ok_or(ParserError::UnexpectedEoF)?;
        let struct_name = match name_token.kind {
            TokenKind::Identifier(name) => name,
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: "Struct name (Identifier)".to_string(),
                    found: format!("{:?}", name_token.kind),
                    line: name_token.line,
                    column: name_token.column,
                });
            }
        };

        self.expect(TokenKind::LeftBrace)?;

        let mut fields = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let field_type = self.parse_type()?;
            let name_token = self.advance().ok_or(ParserError::UnexpectedEoF)?;
            let field_name = match name_token.kind {
                TokenKind::Identifier(name) => name,
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        expected: "Field name (Identifier)".to_string(),
                        found: format!("{:?}", name_token.kind),
                        line: name_token.line,
                        column: name_token.column,
                    });
                }
            };

            self.expect(TokenKind::Semicolon)?;

            fields.push(Parameter {
                name: field_name,
                param_typ: field_type,
            });
        }

        self.expect(TokenKind::RightBrace)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(AstNode::StructDecl(StructDeclData {
            name: struct_name,
            fields,
            line: name_token.line,
            column: name_token.column,
        }))
    }

    fn parse_function(&mut self) -> Result<AstNode<'a>, ParserError> {
        let return_type = self.parse_type()?;

        let name_token = self.advance().ok_or(ParserError::UnexpectedEoF)?;
        let name = match name_token.kind {
            TokenKind::Identifier(name) => name,
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: "Function name (Identifier)".to_string(),
                    found: format!("{:?}", name_token.kind),
                    line: name_token.line,
                    column: name_token.column,
                });
            }
        };

        self.expect(TokenKind::LeftParen)?;

        let mut parameter = Vec::new();

        while !self.check(TokenKind::RightParen) && !self.is_at_end() {
            let field_type = self.parse_type()?;

            let (field_name, _, _) = self.consume_identifier()?;

            parameter.push(Parameter {
                name: field_name,
                param_typ: field_type,
            });

            if self.check(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenKind::RightParen)?;

        let body = self.parse_block()?;

        Ok(AstNode::FunctionDecl(FunctionDeclData {
            name,
            return_type,
            parameter,
            body,
            line: name_token.line,
            column: name_token.column,
        }))
    }

    fn consume_identifier(&mut self) -> Result<(&'a str, usize, usize), ParserError> {
        let token = self.advance().ok_or(ParserError::UnexpectedEoF)?;
        match token.kind {
            TokenKind::Identifier(name) => Ok((name, token.line, token.column)),
            _ => Err(ParserError::UnexpectedToken {
                expected: "Identifier".to_string(),
                found: format!("{:?}", token.kind),
                line: token.line,
                column: token.column,
            }),
        }
    }

    fn parse_block(&mut self) -> Result<Box<AstNode<'a>>, ParserError> {
        let open_brace = self.expect(TokenKind::LeftBrace)?;

        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RightBrace)?;
        Ok(Box::new(AstNode::BlockStatement(BlockStatementData {
            statements,
            line: open_brace.line,
            column: open_brace.column,
        })))
    }

    fn parse_statement(&mut self) -> Result<AstNode<'a>, ParserError> {
        let token = self.peek().ok_or(ParserError::UnexpectedEoF)?;

        match &token.kind {
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::PrintKeyword => self.parse_print_statement(),

            TokenKind::IntKeyword
            | TokenKind::BoolKeyword
            | TokenKind::FloatKeyword
            | TokenKind::StringKeyword => self.parse_var_decl(),

            TokenKind::LeftBrace => {
                let block = self.parse_block()?;
                Ok(*block)
            }
            TokenKind::Identifier(_) => {
                // Wenn nach dem Identifier ein anderer Identifier kommt -> Variablen-Deklaration (z.B. MyStruct x;)
                if matches!(
                    self.tokens.get(self.cursor + 1).map(|t| &t.kind),
                    Some(TokenKind::Identifier(_))
                ) {
                    self.parse_var_decl()
                } else {
                    self.parse_expression_or_assignment()
                }
            }
            other => Err(ParserError::UnexpectedToken {
                expected: "Statement (return, if, while, variable declaration, identifier, etc.)"
                    .to_string(),
                found: format!("{:?}", other),
                line: token.line,
                column: token.column,
            }),
        }
    }

    fn parse_expression_or_assignment(&mut self) -> Result<AstNode<'a>, ParserError> {
        let expr = self.parse_expression()?;

        if self.check(TokenKind::Equal) {
            let eq_token = self.expect(TokenKind::Equal)?;
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;

            Ok(AstNode::AssignmentStatement(AssignmentData {
                target: Box::new(expr),
                value: Box::new(value),
                line: eq_token.line,
                column: eq_token.column,
            }))
        } else {
            self.expect(TokenKind::Semicolon)?;
            Ok(expr)
        }
    }

    fn parse_var_decl(&mut self) -> Result<AstNode<'a>, ParserError> {
        let type_token = self.peek().ok_or(ParserError::UnexpectedEoF)?;

        let var_type = self.parse_type()?;

        let (var_name, _, _) = self.consume_identifier()?;

        let initializer = if self.check(TokenKind::Equal) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(AstNode::VarDeclStatement(VarDeclData {
            name: var_name,
            var_typ: var_type,
            initializer,
            line: type_token.line,
            column: type_token.column,
        }))
    }

    fn parse_if_statement(&mut self) -> Result<AstNode<'a>, ParserError> {
        let if_token = self.expect(TokenKind::If)?;

        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;

        let then_branch = self.parse_block()?;

        let else_branch = if self.check(TokenKind::Else) {
            self.advance();

            if self.check(TokenKind::If) {
                Some(Box::new(self.parse_if_statement()?))
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(AstNode::IfElseStatement(IfElseData {
            condition_expr: Box::new(condition),
            if_branch: then_branch,
            else_branch,
            line: if_token.line,
            column: if_token.column,
        }))
    }

    fn parse_return_statement(&mut self) -> Result<AstNode<'a>, ParserError> {
        let return_token = self.expect(TokenKind::Return)?;

        let value = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(AstNode::ReturnStatement(ReturnData {
            value,
            line: return_token.line,
            column: return_token.column,
        }))
    }

    fn parse_factor(&mut self) -> Result<AstNode<'a>, ParserError> {
        let token = self.advance().ok_or(ParserError::UnexpectedEoF)?;

        let mut node = match &token.kind {
            TokenKind::IntNumber(raw_val) => {
                let val = raw_val
                    .parse::<i16>()
                    .map_err(|_| ParserError::InvalidIntLiteral {
                        raw: raw_val.to_string(),
                        line: token.line,
                        column: token.column,
                    })?;
                Ok(AstNode::IntLiteralExpr(val))
            }
            TokenKind::FloatNumber(raw_val) => {
                let val = raw_val
                    .parse::<f64>()
                    .map_err(|_| ParserError::InvalidFloatLiteral {
                        raw: raw_val.to_string(),
                        line: token.line,
                        column: token.column,
                    })?;
                Ok(AstNode::FloatLiteralExpr(val))
            }
            TokenKind::String(val) => Ok(AstNode::StringLiteralExpr(val)),
            TokenKind::Bool(true) => Ok(AstNode::BoolLiteralExpr(true)),
            TokenKind::Bool(false) => Ok(AstNode::BoolLiteralExpr(false)),
            TokenKind::LeftParen => {
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                Ok(expr)
            }

            TokenKind::Identifier(name) => {
                if self.check(TokenKind::LeftParen) {
                    self.parse_call_expr(name, token.line, token.column)
                } else {
                    Ok(AstNode::VariableExpr(name))
                }
            }

            other => Err(ParserError::UnexpectedToken {
                expected: "literal, identifier, or '('".to_string(),
                found: format!("{:?}", other),
                line: token.line,
                column: token.column,
            }),
        }?;

        // Postfix-Operatoren parsen: .member und [index]
        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Dot => {
                    let dot_token = self.advance().unwrap();
                    let (member_name, _, _) = self.consume_identifier()?;
                    node = AstNode::MemberAccessExpr(MemberAccessData {
                        object: Box::new(node),
                        member: member_name.to_string(),
                        line: dot_token.line,
                        column: dot_token.column,
                    });
                }
                TokenKind::LeftBracket => {
                    let bracket_token = self.advance().unwrap();
                    let index_expr = self.parse_expression()?;
                    self.expect(TokenKind::RightBracket)?;
                    node = AstNode::ArrayIndexExpr(ArrayIndexData {
                        array: Box::new(node),
                        index: Box::new(index_expr),
                        line: bracket_token.line,
                        column: bracket_token.column,
                    });
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_term(&mut self) -> Result<AstNode<'a>, ParserError> {
        let mut left = self.parse_factor()?;
        while let Some(token) = self.peek() {
            let operator = match token.kind {
                TokenKind::Star => BinaryOperator::Mul,
                TokenKind::Slash => BinaryOperator::Div,
                _ => break,
            };
            let line = token.line;
            let column = token.column;
            self.advance();

            let right = self.parse_factor()?;
            left = AstNode::BinaryExpr(BinaryExpData {
                left: Box::new(left),
                right: Box::new(right),
                operator,
                line,
                column,
            });
        }
        Ok(left)
    }

    fn parse_call_expr(
        &mut self,
        callee: &'a str,
        line: usize,
        column: usize,
    ) -> Result<AstNode<'a>, ParserError> {
        self.expect(TokenKind::LeftParen)?;

        let mut args = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_expression()?);

                if self.check(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RightParen)?;

        Ok(AstNode::CallExpr(CallData {
            name: callee,
            arguments: args,
            line,
            column,
        }))
    }

    fn parse_comparison(&mut self) -> Result<AstNode<'a>, ParserError> {
        let mut left = self.parse_additive_expr()?;

        while let Some(token) = self.peek() {
            let op = match token.kind {
                TokenKind::DoubleEqual => BinaryOperator::DoubleEqual,
                //TODO: TokenKind::NotEqual => BinaryOperator::NotEqual,
                TokenKind::LessThan => BinaryOperator::LessThan,
                TokenKind::LessOrEqual => BinaryOperator::LessOrEqual,
                TokenKind::GreaterThan => BinaryOperator::GreaterThan,
                TokenKind::GreaterOrEqual => BinaryOperator::GreaterOrEqual,
                _ => break,
            };

            let line = token.line;
            let column = token.column;
            self.advance();

            let right = self.parse_additive_expr()?;

            left = AstNode::BinaryExpr(BinaryExpData {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                line,
                column,
            });
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<AstNode<'a>, ParserError> {
        let mut left = self.parse_term()?;

        while let Some(token) = self.peek() {
            let op = match token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Sub,
                _ => break,
            };

            let line = token.line;
            let column = token.column;
            self.advance();

            let right = self.parse_term()?;

            left = AstNode::BinaryExpr(BinaryExpData {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                line,
                column,
            });
        }

        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<AstNode<'a>, ParserError> {
        self.parse_comparison()
    }

    fn parse_type(&mut self) -> Result<Type<'a>, ParserError> {
        let token = self.advance().ok_or(ParserError::UnexpectedEoF)?;

        let mut base_type = match token.kind {
            TokenKind::IntKeyword => Type::Int,
            TokenKind::BoolKeyword => Type::Bool,
            TokenKind::FloatKeyword => Type::Float,
            TokenKind::StringKeyword => Type::String,
            TokenKind::Identifier(name) => Type::Struct(name),
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: "type keyword or struct name".to_string(),
                    found: format!("{:?}", token.kind),
                    line: token.line,
                    column: token.column,
                });
            }
        };

        if self.check(TokenKind::LeftBracket) {
            self.advance();
            self.expect(TokenKind::RightBracket)?;
            base_type = Type::Array(Box::new(base_type));
        }

        Ok(base_type)
    }

    fn parse_while_statement(&mut self) -> Result<AstNode<'a>, ParserError> {
        let while_token = self.expect(TokenKind::While)?;

        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;

        let body = self.parse_block()?;

        Ok(AstNode::WhileStatement(WhileLoopData {
            condition_expr: Box::new(condition),
            body,
            line: while_token.line,
            column: while_token.column,
        }))
    }

    fn parse_print_statement(&mut self) -> Result<AstNode<'a>, ParserError> {
        self.expect(TokenKind::PrintKeyword)?;

        self.expect(TokenKind::LeftParen)?;
        let expr = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(AstNode::PrintStatement(Box::new(expr)))
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
