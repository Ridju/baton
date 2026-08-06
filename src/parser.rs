use std::slice::Iter;
use std::{iter::Peekable, os::macos::raw::stat};

use crate::parser::ParserError::{NotImplemented, UnexpectedEoF};
use crate::scanner::Token;

#[derive(Debug, PartialEq)]
pub enum ParserError {
    UnexpectedToken { expected: String, found: String },
    UnknownType(String),
    NotImplemented(String),
    InvalidIntLiteral(String),
    UnexpectedEoF,
}

#[derive(Debug, PartialEq)]
enum Type {
    Int,
}

impl Type {
    fn get_from_token(token: &Token) -> Result<Type, ParserError> {
        match token {
            Token::IntKeyword => Ok(Type::Int),
            other => Err(ParserError::UnknownType(format!("{:?}", other))),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Parameter {
    name: String,
    param_typ: Type,
}

#[derive(Debug, PartialEq)]
struct FunctionDeclData {
    name: String,
    return_type: Type,
    parameter: Vec<Parameter>,
    body: Box<AstNode>,
}

#[derive(Debug, PartialEq)]
enum AstNode {
    Programm(Vec<AstNode>),
    FunctionDecl(FunctionDeclData),
    ReturnStatement(Box<AstNode>),
    IntLiteralExpr(i16),
    BlockStatement(Vec<AstNode>),
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
                Token::IntKeyword => match self.tokens.clone().nth(2) {
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
                        expected: "int".to_string(),
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

    fn parse_return_statement(&mut self) -> Result<AstNode, ParserError> {
        self.tokens.next();

        let expr_node = match self.tokens.peek() {
            Some(Token::IntNumber(number)) => {
                let num_str = number.clone();
                self.tokens.next();
                let num = number
                    .parse::<i16>()
                    .map_err(|_| ParserError::InvalidIntLiteral(num_str))?;
                AstNode::IntLiteralExpr(num)
            }
            Some(Token::Semicolon) => AstNode::IntLiteralExpr(0),
            Some(other) => {
                return Err(ParserError::UnexpectedToken {
                    expected: "int literal or ';'".to_string(),
                    found: format!("{:?}", other),
                });
            }
            None => return Err(ParserError::UnexpectedEoF),
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
                expected: "int".to_string(),
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
            // Cuts off before parameter name or closing paren
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
            // Missing RightBrace at the end
        ];

        let result = Parser::new(&tokens).parse();
        assert_eq!(result, Err(ParserError::UnexpectedEoF));
    }

    #[test]
    fn test_parser_unknown_parameter_type() {
        let tokens = vec![
            Token::IntKeyword,
            Token::Identifier("foo".to_string()),
            Token::LeftParen,
            Token::Identifier("float".to_string()), // Unsupported type token
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::RightBrace,
        ];

        let result = Parser::new(&tokens).parse();
        assert!(matches!(result, Err(ParserError::UnknownType(_))));
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
            Token::LeftParen, // Invalid token directly after return instead of number or semicolon
            Token::Semicolon,
            Token::RightBrace,
        ];

        let result = Parser::new(&tokens).parse();
        assert_eq!(
            result,
            Err(ParserError::UnexpectedToken {
                expected: "int literal or ';'".to_string(),
                found: "LeftParen".to_string(),
            })
        );
    }
}
