use std::fmt::Alignment::Right;
use std::iter::Peekable;
use std::slice::Iter;

use scanner::TokenKind;

use crate::parser::AstNode::BinaryExpr;
use crate::parser::ParserError::{NotImplemented, UnexpectedEoF};
use scanner::Token;

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
    InvalidIntLiteral(String),
    InvalidFloatLiteral(String),
    UnexpectedEoF,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Bool,
    Float,
    String,
    Struct(String),
    Array(Box<Type>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_typ: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclData {
    pub name: String,
    pub return_type: Type,
    pub parameter: Vec<Parameter>,
    pub body: Box<AstNode>,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpData {
    pub left: Box<AstNode>,
    pub right: Box<AstNode>,
    pub operator: BinaryOperator,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDeclData {
    pub name: String,
    pub var_typ: Type,
    pub initializer: Option<Box<AstNode>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AssignmentData {
    pub target: Box<AstNode>,
    pub value: Box<AstNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfElseData {
    pub condition_expr: Box<AstNode>,
    pub if_branch: Box<AstNode>,
    pub else_branch: Option<Box<AstNode>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CallData {
    pub name: String,
    pub arguments: Vec<AstNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MemberAccessData {
    pub object: Box<AstNode>,
    pub member: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayIndexData {
    pub array: Box<AstNode>,
    pub index: Box<AstNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhileLoopData {
    pub condition_expr: Box<AstNode>,
    pub body: Box<AstNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode<'a> {
    Programm(Vec<AstNode<'a>>),
    FunctionDecl(FunctionDeclData),
    ReturnStatement(Box<AstNode<'a>>),
    IntLiteralExpr(i16),
    BoolLiteralExpr(bool),
    BlockStatement(Vec<AstNode<'a>>),
    BinaryExpr(BinaryExpData),
    IfElseStatement(IfElseData),
    VariableExpr(String),
    VarDeclStatement(VarDeclData),
    AssignmentStatement(AssignmentData),
    CallExpr(CallData),
    FloatLiteralExpr(f64),
    StringLiteralExpr(String),
    StructDecl {
        name: &'a str,
        fields: Vec<Parameter>,
        line: usize,
        column: usize,
    },
    MemberAccessExpr(MemberAccessData),
    ArrayIndexExpr(ArrayIndexData),
    WhileStatement(WhileLoopData),
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

    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) -> Option<&Token<'a>> {
        let token = self.tokens.get(self.cursor);
        self.cursor += 1;
        token
    }

    fn expect(&mut self, expected: TokenKind) -> Result<&Token<'a>, ParserError> {
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

    pub fn parse(&mut self) -> Result<AstNode, ParserError> {
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

    fn parse_struct_decl(&mut self) -> Result<AstNode, ParserError> {
        self.expect(TokenKind::StructKeyword)?;

        let name_token = self.advance().ok_or(ParserError::UnexpectedEoF)?;
        let struct_name = match &name_token.kind {
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

        self.expect(TokenKind::LeftParen)?;

        let mut fields = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let field_type = self.parse_type()?;
            let field_name = self.parse_identifier()?;
            self.expect(TokenKind::Semicolon)?;
            fields.push(Parameter {
                name: field_name,
                param_typ: field_type,
            });
        }

        self.expect(TokenKind::RightBrace)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(AstNode::StructDecl {
            name: struct_name,
            fields,
            line: name_token.line,
            column: name_token.column,
        })
    }

    fn parse_function(&mut self) -> Result<AstNode, ParserError> {
        let return_type = self.parse_type()?;
        let name = match self.tokens.next() {
            Some(Token::Identifier(name)) => name.clone(),
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "identifier (function name)".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        match self.tokens.next() {
            Some(Token::LeftParen) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "'('".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        }

        let mut parameter = Vec::new();

        loop {
            if let Some(Token::RightParen) = self.tokens.peek() {
                self.tokens.next();
                break;
            }

            let param_type = self.parse_type()?;

            let param_name = match self.tokens.next() {
                Some(Token::Identifier(name)) => name.clone(),
                Some(other) => {
                    return Err(ParserError::UnexpectedToken {
                        expected: "identifier (paramter name)".to_string(),
                        found: format!("{:?}", other),
                    });
                }
                None => return Err(ParserError::UnexpectedEoF),
            };

            parameter.push(Parameter {
                name: param_name,
                param_typ: param_type,
            });

            match self.tokens.peek() {
                Some(Token::RightParen) => {
                    self.tokens.next();
                    break;
                }
                Some(Token::Comma) => {
                    self.tokens.next();
                }
                Some(other) => {
                    return Err(ParserError::UnexpectedToken {
                        expected: "')' or ','".to_string(),
                        found: format!("{:?}", other),
                    });
                }
                None => return Err(ParserError::UnexpectedEoF),
            }
        }

        let body = self.parse_block()?;

        Ok(AstNode::FunctionDecl(FunctionDeclData {
            name,
            return_type,
            parameter,
            body,
        }))
    }

    fn parse_block(&mut self) -> Result<Box<AstNode>, ParserError> {
        match self.tokens.next() {
            Some(Token::LeftBrace) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "'{'".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        }

        let mut statements = Vec::new();

        loop {
            match self.tokens.peek() {
                Some(Token::RightBrace) => {
                    self.tokens.next();
                    break;
                }
                Some(Token::Return) => {
                    statements.push(self.parse_return_statement()?);
                }
                Some(Token::PrintKeyword) => {
                    statements.push(self.parse_print_statement()?);
                }
                Some(Token::IntKeyword)
                | Some(Token::BoolKeyword)
                | Some(Token::FloatKeyword)
                | Some(Token::StringKeyword) => {
                    statements.push(self.parse_var_decl()?);
                }
                Some(Token::Identifier(_)) => {
                    let mut clone = self.tokens.clone();
                    clone.next();
                    let is_var_decl = matches!(clone.next(), Some(Token::Identifier(_)));

                    if is_var_decl {
                        statements.push(self.parse_var_decl()?);
                    } else {
                        statements.push(self.parse_assignment()?);
                    }
                }
                Some(Token::If) => {
                    statements.push(self.parse_if_statement()?);
                }
                Some(Token::While) => {
                    statements.push(self.parse_while_statement()?);
                }
                Some(other) => {
                    return Err(ParserError::UnexpectedToken {
                        expected: "return statment or '}'".to_string(),
                        found: format!("{:?}", other),
                    });
                }
                None => return Err(ParserError::UnexpectedEoF),
            }
        }

        Ok(Box::new(AstNode::BlockStatement(statements)))
    }

    fn parse_var_decl(&mut self) -> Result<AstNode, ParserError> {
        let var_typ = self.parse_type()?;
        let name = match self.tokens.next() {
            Some(Token::Identifier(name)) => name.clone(),
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        let initializer = match self.tokens.peek() {
            Some(Token::Equal) => {
                self.tokens.next();
                Some(Box::new(self.parse_expression()?))
            }
            _ => None,
        };

        match self.tokens.next() {
            Some(Token::Semicolon) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "';'".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        }

        Ok(AstNode::VarDeclStatement(VarDeclData {
            name,
            var_typ,
            initializer,
        }))
    }

    fn parse_if_statement(&mut self) -> Result<AstNode, ParserError> {
        self.tokens.next();

        match self.tokens.next() {
            Some(Token::LeftParen) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "'('".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        let comparison = self.parse_comparison()?;

        match self.tokens.next() {
            Some(Token::RightParen) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "')'".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        let if_branch = self.parse_block()?;

        let else_branch = if let Some(Token::Else) = self.tokens.peek() {
            self.tokens.next();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(AstNode::IfElseStatement(IfElseData {
            condition_expr: Box::new(comparison),
            if_branch,
            else_branch,
        }))
    }

    fn parse_return_statement(&mut self) -> Result<AstNode, ParserError> {
        self.tokens.next();

        let expr_node = if let Some(Token::Semicolon) = self.tokens.peek() {
            AstNode::IntLiteralExpr(0)
        } else {
            self.parse_expression()?
        };
        match self.tokens.next() {
            Some(Token::Semicolon) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "';'".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        }

        Ok(AstNode::ReturnStatement(Box::new(expr_node)))
    }

    fn parse_assignment(&mut self) -> Result<AstNode, ParserError> {
        let target = self.parse_factor()?;

        match self.tokens.next() {
            Some(Token::Equal) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "'='".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        }

        let value = Box::new(self.parse_expression()?);

        match self.tokens.next() {
            Some(Token::Semicolon) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "';'".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        }

        Ok(AstNode::AssignmentStatement(AssignmentData {
            target: Box::new(target),
            value,
        }))
    }

    fn parse_factor(&mut self) -> Result<AstNode, ParserError> {
        let mut expr = match self.tokens.next() {
            Some(Token::IntNumber(num_str)) => {
                let number = num_str.clone();
                let num = number
                    .parse::<i16>()
                    .map_err(|_| ParserError::InvalidIntLiteral(number))?;
                AstNode::IntLiteralExpr(num)
            }
            Some(Token::Bool(val)) => AstNode::BoolLiteralExpr(*val),
            Some(Token::FloatNumber(num_str)) => {
                let num = num_str
                    .parse::<f64>()
                    .map_err(|_| ParserError::InvalidFloatLiteral(num_str.clone()))?;
                AstNode::FloatLiteralExpr(num)
            }
            Some(Token::String(val)) => AstNode::StringLiteralExpr(val.clone()),
            Some(Token::Identifier(name)) => {
                if let Some(Token::LeftParen) = self.tokens.peek() {
                    let mut args = Vec::new();
                    self.tokens.next();
                    loop {
                        if let Some(Token::RightParen) = self.tokens.peek() {
                            self.tokens.next();
                            break;
                        }
                        let arg = self.parse_expression()?;
                        args.push(arg);

                        match self.tokens.peek() {
                            Some(Token::RightParen) => {
                                self.tokens.next();
                                break;
                            }
                            Some(Token::Comma) => {
                                self.tokens.next();
                            }
                            Some(other) => {
                                return Err(ParserError::UnexpectedToken {
                                    expected: "',', or ')'".to_string(),
                                    found: format!("{:?}", other),
                                });
                            }
                            None => return Err(ParserError::UnexpectedEoF),
                        }
                    }

                    AstNode::CallExpr(CallData {
                        name: name.clone(),
                        arguments: args,
                    })
                } else {
                    AstNode::VariableExpr(name.clone())
                }
            }
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "Int Number".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        loop {
            match self.tokens.peek() {
                Some(Token::Dot) => {
                    self.tokens.next();
                    let member = match self.tokens.next() {
                        Some(Token::Identifier(name)) => name.clone(),
                        Some(other) => {
                            return Err(ParserError::UnexpectedToken {
                                expected: "identifier (field name)".to_string(),
                                found: format!("{:?}", other),
                            });
                        }
                        None => return Err(ParserError::UnexpectedEoF),
                    };
                    expr = AstNode::MemberAccessExpr(MemberAccessData {
                        object: Box::new(expr),
                        member,
                    });
                }
                Some(Token::LeftBracket) => {
                    self.tokens.next();
                    let index = self.parse_expression()?;
                    match self.tokens.next() {
                        Some(Token::RightBracket) => {}
                        Some(other) => {
                            return Err(ParserError::UnexpectedToken {
                                expected: "']'".to_string(),
                                found: format!("{:?}", other),
                            });
                        }
                        None => return Err(ParserError::UnexpectedEoF),
                    };
                    expr = AstNode::ArrayIndexExpr(ArrayIndexData {
                        array: Box::new(expr),
                        index: Box::new(index),
                    });
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<AstNode, ParserError> {
        let mut left = self.parse_factor()?;
        loop {
            let operator = match self.tokens.peek() {
                Some(Token::Star) => {
                    self.tokens.next();
                    BinaryOperator::Mul
                }
                Some(Token::Slash) => {
                    self.tokens.next();
                    BinaryOperator::Div
                }
                _ => break,
            };
            let right = self.parse_factor()?;
            left = AstNode::BinaryExpr(BinaryExpData {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            })
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<AstNode, ParserError> {
        let mut left = self.parse_additive_expr()?;
        loop {
            let operator = match self.tokens.peek() {
                Some(Token::DoubleEqual) => {
                    self.tokens.next();
                    BinaryOperator::DoubleEqual
                }
                Some(Token::LessThan) => {
                    self.tokens.next();
                    BinaryOperator::LessThan
                }
                Some(Token::LessOrEqual) => {
                    self.tokens.next();
                    BinaryOperator::LessOrEqual
                }
                Some(Token::GreaterThan) => {
                    self.tokens.next();
                    BinaryOperator::GreaterThan
                }
                Some(Token::GreaterOrEqual) => {
                    self.tokens.next();
                    BinaryOperator::GreaterOrEqual
                }
                _ => break,
            };

            let right = self.parse_additive_expr()?;
            left = AstNode::BinaryExpr(BinaryExpData {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            })
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<AstNode, ParserError> {
        let mut left = self.parse_term()?;
        loop {
            let operator = match self.tokens.peek() {
                Some(Token::Plus) => {
                    self.tokens.next();
                    BinaryOperator::Add
                }
                Some(Token::Minus) => {
                    self.tokens.next();
                    BinaryOperator::Sub
                }
                _ => break,
            };
            let right = self.parse_term()?;
            left = AstNode::BinaryExpr(BinaryExpData {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            })
        }
        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<AstNode, ParserError> {
        self.parse_comparison()
    }

    fn parse_type(&mut self) -> Result<Type, ParserError> {
        let mut base_type = match self.tokens.next() {
            Some(Token::IntKeyword) => Type::Int,
            Some(Token::BoolKeyword) => Type::Bool,
            Some(Token::FloatKeyword) => Type::Float,
            Some(Token::StringKeyword) => Type::String,
            Some(Token::Identifier(name)) => Type::Struct(name.clone()),
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "type keyword or struct name".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        if let Some(Token::LeftBracket) = self.tokens.peek() {
            self.tokens.next();
            match self.tokens.next() {
                Some(Token::RightBracket) => {
                    base_type = Type::Array(Box::new(base_type));
                }
                Some(other) => {
                    return Err(ParserError::UnexpectedToken {
                        expected: "']'".to_string(),
                        found: format!("{:?}", other),
                    });
                }
                None => return Err(ParserError::UnexpectedEoF),
            }
        }
        Ok(base_type)
    }

    fn parse_while_statement(&mut self) -> Result<AstNode, ParserError> {
        self.tokens.next();

        match self.tokens.next() {
            Some(Token::LeftParen) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "'('".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        let condition = self.parse_comparison()?;

        match self.tokens.next() {
            Some(Token::RightParen) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "'('".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        let body = self.parse_block()?;

        Ok(AstNode::WhileStatement(WhileLoopData {
            condition_expr: Box::new(condition),
            body,
        }))
    }

    fn parse_print_statement(&mut self) -> Result<AstNode, ParserError> {
        self.tokens.next();

        match self.tokens.next() {
            Some(Token::LeftParen) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "'('".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        let expr = self.parse_expression()?;

        match self.tokens.next() {
            Some(Token::RightParen) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "'('".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        match self.tokens.next() {
            Some(Token::Semicolon) => {}
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "'('".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        };

        Ok(AstNode::PrintStatement(Box::new(expr)))
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
