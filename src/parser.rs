use std::slice::Iter;
use std::{iter::Peekable, os::macos::raw::stat};

use crate::scanner::Token;

#[derive(Debug, PartialEq)]
enum Type {
    Int,
}

impl Type {
    fn get_from_token(token: &Token) -> Type {
        match token {
            Token::IntKeyword => Type::Int,
            _ => todo!("Implement Type Error"),
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

    pub fn parse(&mut self) -> AstNode {
        let mut nodes = Vec::new();
        while let Some(token) = self.tokens.peek() {
            match token {
                Token::IntKeyword => match self.tokens.clone().nth(2) {
                    Some(Token::LeftParen) => {
                        nodes.push(self.parse_function());
                    }
                    _ => {
                        todo!("Implement global variable parsing");
                    }
                },
                _ => {
                    todo!("not implemented");
                }
            }
        }

        AstNode::Programm(nodes)
    }

    fn parse_function(&mut self) -> AstNode {
        let return_type = Type::get_from_token(self.tokens.next().unwrap());
        let name = match self.tokens.next() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => todo!("Add error handling to function name"),
        };

        match self.tokens.next() {
            Some(Token::LeftParen) => {}
            _ => todo!("Add error handling"),
        }

        let mut parameter = Vec::new();

        loop {
            if let Some(Token::RightParen) = self.tokens.peek() {
                self.tokens.next();
                break;
            }

            let param_type = Type::get_from_token(self.tokens.next().unwrap());

            let param_name = match self.tokens.next() {
                Some(Token::Identifier(name)) => name.clone(),
                _ => {
                    todo!("add error handling");
                }
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
                _ => {
                    todo!("Add error handling");
                }
            }
        }

        let body = self.parse_block();

        AstNode::FunctionDecl(FunctionDeclData {
            name,
            return_type,
            parameter,
            body,
        })
    }

    fn parse_block(&mut self) -> Box<AstNode> {
        self.tokens.next();
        let mut statements = Vec::new();

        loop {
            match self.tokens.peek() {
                Some(Token::RightBrace) => {
                    self.tokens.next();
                    break;
                }
                Some(Token::Return) => {
                    statements.push(self.parse_return_statement());
                }
                _ => {
                    todo!("Add error handling");
                }
            }
        }

        Box::new(AstNode::BlockStatement(statements))
    }

    fn parse_return_statement(&mut self) -> AstNode {
        self.tokens.next();

        let expr_node = match self.tokens.peek() {
            Some(Token::IntNumber(number)) => {
                self.tokens.next();
                let num = number.parse::<i16>().unwrap();
                AstNode::IntLiteralExpr(num)
            }
            Some(Token::Semicolon) => AstNode::IntLiteralExpr(0),
            _ => {
                todo!("Add error handling");
            }
        };

        match self.tokens.next() {
            Some(Token::Semicolon) => {}
            _ => {
                todo!("Add error handling");
            }
        }

        AstNode::ReturnStatement(Box::new(expr_node))
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

        let ast = parse(tokens);

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
}
