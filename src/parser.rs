use std::fmt::Alignment::Right;
use std::iter::Peekable;
use std::slice::Iter;

use crate::parser::AstNode::BinaryExpr;
use crate::parser::ParserError::{NotImplemented, UnexpectedEoF};
use crate::scanner::Token;

#[derive(Debug, PartialEq)]
pub enum ParserError {
    UnexpectedToken { expected: String, found: String },
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
}

impl Type {
    fn get_from_token(token: &Token) -> Result<Type, ParserError> {
        match token {
            Token::IntKeyword => Ok(Type::Int),
            Token::BoolKeyword => Ok(Type::Bool),
            Token::FloatKeyword => Ok(Type::Float),
            Token::StringKeyword => Ok(Type::String),
            other => Err(ParserError::UnexpectedToken {
                expected: "'int', 'bool', 'float', 'string'".to_string(),
                found: format!("{:?}", other),
            }),
        }
    }
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
    pub name: String,
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
pub enum AstNode {
    Programm(Vec<AstNode>),
    FunctionDecl(FunctionDeclData),
    ReturnStatement(Box<AstNode>),
    IntLiteralExpr(i16),
    BoolLiteralExpr(bool),
    BlockStatement(Vec<AstNode>),
    BinaryExpr(BinaryExpData),
    IfElseStatement(IfElseData),
    VariableExpr(String),
    VarDeclStatement(VarDeclData),
    AssignmentStatement(AssignmentData),
    CallExpr(CallData),
    FloatLiteralExpr(f64),
    StringLiteralExpr(String),
}

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<AstNode, ParserError> {
        let mut nodes = Vec::new();
        while let Some(token) = self.tokens.peek() {
            match token {
                Token::IntKeyword
                | Token::BoolKeyword
                | Token::FloatKeyword
                | Token::StringKeyword => match self.tokens.clone().nth(2) {
                    Some(Token::LeftParen) => {
                        nodes.push(self.parse_function()?);
                    }
                    _ => {
                        return Err(ParserError::NotImplemented(
                            "Global variable parsing not implemented".to_string(),
                        ));
                    }
                },
                other => {
                    return Err(ParserError::UnexpectedToken {
                        expected: "type keyword (int, bool, float, string)".to_string(),
                        found: format!("{:?}", other),
                    });
                }
            }
        }

        Ok(AstNode::Programm(nodes))
    }

    fn parse_function(&mut self) -> Result<AstNode, ParserError> {
        let return_type = Type::get_from_token(self.tokens.next().unwrap())?;
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

            let param_type = Type::get_from_token(self.tokens.next().unwrap())?;

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
                Some(Token::IntKeyword)
                | Some(Token::BoolKeyword)
                | Some(Token::FloatKeyword)
                | Some(Token::StringKeyword) => {
                    statements.push(self.parse_var_decl()?);
                }
                Some(Token::Identifier(_)) => {
                    let mut clone = self.tokens.clone();
                    clone.next();
                    match clone.peek() {
                        Some(Token::Equal) => {
                            statements.push(self.parse_assignment()?);
                        }
                        Some(other) => {
                            return Err(ParserError::UnexpectedToken {
                                expected: "'='".to_string(),
                                found: format!("{:?}", other),
                            });
                        }
                        None => return Err(ParserError::UnexpectedEoF),
                    }
                }
                Some(Token::If) => {
                    statements.push(self.parse_if_statement()?);
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
        let var_typ = Type::get_from_token(self.tokens.next().unwrap())?;
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
        let name = match self.tokens.next() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => unreachable!(),
        };

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

        Ok(AstNode::AssignmentStatement(AssignmentData { name, value }))
    }

    fn parse_factor(&mut self) -> Result<AstNode, ParserError> {
        match self.tokens.next() {
            Some(Token::IntNumber(num_str)) => {
                let number = num_str.clone();
                let num = number
                    .parse::<i16>()
                    .map_err(|_| ParserError::InvalidIntLiteral(number))?;
                Ok(AstNode::IntLiteralExpr(num))
            }
            Some(Token::Bool(val)) => Ok(AstNode::BoolLiteralExpr(*val)),
            Some(Token::FloatNumber(num_str)) => {
                let num = num_str
                    .parse::<f64>()
                    .map_err(|_| ParserError::InvalidFloatLiteral(num_str.clone()))?;
                Ok(AstNode::FloatLiteralExpr(num))
            }
            Some(Token::String(val)) => Ok(AstNode::StringLiteralExpr(val.clone())),
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

                    Ok(AstNode::CallExpr(CallData {
                        name: name.clone(),
                        arguments: args,
                    }))
                } else {
                    Ok(AstNode::VariableExpr(name.clone()))
                }
            }
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "Int Number".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Token;

    #[test]
    fn test_parse_simple_function() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("42".to_string()),
            Token::Semicolon,
            Token::RightBrace,
        ];

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(42)),
            )])),
        })]);

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_simple_function_with_paramter() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::IntKeyword,
            Token::Identifier("a".to_string()),
            Token::Comma,
            Token::IntKeyword,
            Token::Identifier("b".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("42".to_string()),
            Token::Semicolon,
            Token::RightBrace,
        ];

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: vec![
                Parameter {
                    name: "a".to_string(),
                    param_typ: Type::Int,
                },
                Parameter {
                    name: "b".to_string(),
                    param_typ: Type::Int,
                },
            ],
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(42)),
            )])),
        })]);

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parser_unexpected_eof() {
        let tokens = vec![Token::IntKeyword, Token::Identifier("main".to_string())];

        let result = Parser::new(&tokens).parse();
        assert_eq!(
            result,
            Err(ParserError::NotImplemented(
                "Global variable parsing not implemented".to_string()
            ))
        );
    }

    #[test]
    fn test_parser_invalid_int_literal() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("999999".to_string()),
            Token::Semicolon,
            Token::RightBrace,
        ];

        let result = Parser::new(&tokens).parse();
        assert_eq!(
            result,
            Err(ParserError::InvalidIntLiteral("999999".to_string()))
        );
    }

    #[test]
    fn test_parser_unexpected_global_token() {
        let tokens = vec![Token::Semicolon];

        let result = Parser::new(&tokens).parse();
        assert_eq!(
            result,
            Err(ParserError::UnexpectedToken {
                expected: "type keyword (int, bool, float, string)".to_string(),
                found: "Semicolon".to_string(),
            })
        );
    }

    #[test]
    fn test_parser_missing_left_brace() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::Return,
            Token::IntNumber("0".to_string()),
            Token::Semicolon,
            Token::RightBrace,
        ];

        let result = Parser::new(&tokens).parse();
        assert_eq!(
            result,
            Err(ParserError::UnexpectedToken {
                expected: "'{'".to_string(),
                found: "Return".to_string(),
            })
        );
    }

    #[test]
    fn test_parser_not_implemented_global_var() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("global_var".to_string()),
            Token::Semicolon,
        ];

        let result = Parser::new(&tokens).parse();
        assert_eq!(
            result,
            Err(ParserError::NotImplemented(
                "Global variable parsing not implemented".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_function_with_parameter() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("add".to_string()),
            Token::LeftParen,
            Token::IntKeyword,
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("10".to_string()),
            Token::Semicolon,
            Token::RightBrace,
        ];

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "add".to_string(),
            return_type: Type::Int,
            parameter: vec![Parameter {
                name: "x".to_string(),
                param_typ: Type::Int,
            }],
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(10)),
            )])),
        })]);

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_empty_return_statement() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::Semicolon,
            Token::RightBrace,
        ];

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(0)),
            )])),
        })]);

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parser_eof_inside_parameters() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::IntKeyword,
        ];

        let result = Parser::new(&tokens).parse();
        assert_eq!(result, Err(ParserError::UnexpectedEoF));
    }

    #[test]
    fn test_parser_eof_inside_block() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("0".to_string()),
            Token::Semicolon,
        ];

        let result = Parser::new(&tokens).parse();
        assert_eq!(result, Err(ParserError::UnexpectedEoF));
    }

    #[test]
    fn test_parser_invalid_return_expression() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::LeftParen,
            Token::Semicolon,
            Token::RightBrace,
        ];

        let result = Parser::new(&tokens).parse();
        assert_eq!(
            result,
            Err(ParserError::UnexpectedToken {
                expected: "Int Number".to_string(),
                found: "LeftParen".to_string(),
            })
        );
    }

    #[test]
    fn test_parse_complex_function_expression() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("compute".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("10".to_string()),
            Token::Plus,
            Token::IntNumber("20".to_string()),
            Token::Star,
            Token::IntNumber("2".to_string()),
            Token::Semicolon,
            Token::RightBrace,
        ];

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "compute".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::BinaryExpr(BinaryExpData {
                    left: Box::new(AstNode::IntLiteralExpr(10)),
                    right: Box::new(AstNode::BinaryExpr(BinaryExpData {
                        left: Box::new(AstNode::IntLiteralExpr(20)),
                        right: Box::new(AstNode::IntLiteralExpr(2)),
                        operator: BinaryOperator::Mul,
                    })),
                    operator: BinaryOperator::Add,
                })),
            )])),
        })]);

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_if_else_statement() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::If,
            Token::LeftParen,
            Token::IntNumber("5".to_string()),
            Token::LessThan,
            Token::IntNumber("10".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("1".to_string()),
            Token::Semicolon,
            Token::RightBrace,
            Token::Else,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("0".to_string()),
            Token::Semicolon,
            Token::RightBrace,
            Token::RightBrace,
        ];

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::IfElseStatement(
                IfElseData {
                    condition_expr: Box::new(AstNode::BinaryExpr(BinaryExpData {
                        left: Box::new(AstNode::IntLiteralExpr(5)),
                        right: Box::new(AstNode::IntLiteralExpr(10)),
                        operator: BinaryOperator::LessThan,
                    })),
                    if_branch: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                        Box::new(AstNode::IntLiteralExpr(1)),
                    )])),
                    else_branch: Some(Box::new(AstNode::BlockStatement(vec![
                        AstNode::ReturnStatement(Box::new(AstNode::IntLiteralExpr(0))),
                    ]))),
                },
            )])),
        })]);

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_if_without_else() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::If,
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::DoubleEqual,
            Token::IntNumber("0".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("42".to_string()),
            Token::Semicolon,
            Token::RightBrace,
            Token::RightBrace,
        ];

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().unwrap();

        if let AstNode::Programm(nodes) = ast {
            if let AstNode::FunctionDecl(func) = &nodes[0] {
                if let AstNode::BlockStatement(stmts) = &*func.body {
                    if let AstNode::IfElseStatement(if_data) = &stmts[0] {
                        assert!(if_data.else_branch.is_none());
                        return;
                    }
                }
            }
        }
        panic!("AST structure did not match expected IfElseStatement layout");
    }
}
