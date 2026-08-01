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

pub fn parse(tokens: Vec<Token>) -> AstNode {
    let mut tokens = tokens.iter().peekable();
    let mut nodes = Vec::new();
    while let Some(token) = tokens.peek() {
        match token {
            Token::IntKeyword => match tokens.clone().nth(2) {
                Some(Token::LeftParen) => {
                    nodes.push(parse_function(&mut tokens));
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

fn parse_function(tokens: &mut Peekable<std::slice::Iter<'_, Token>>) -> AstNode {
    let return_type = Type::get_from_token(tokens.next().unwrap());
    let name = match tokens.next() {
        Some(Token::Identifier(name)) => name.clone(),
        _ => todo!("Add error handling to function name"),
    };

    match tokens.next() {
        Some(Token::LeftParen) => {}
        _ => todo!("Add error handling"),
    }

    let mut parameter = Vec::new();

    loop {
        if let Some(Token::RightParen) = tokens.peek() {
            tokens.next();
            break;
        }

        let param_type = Type::get_from_token(tokens.next().unwrap());

        let param_name = match tokens.next() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => {
                todo!("add error handling");
            }
        };

        parameter.push(Parameter {
            name: param_name,
            param_typ: param_type,
        });

        match tokens.peek() {
            Some(Token::RightParen) => {
                tokens.next();
                break;
            }
            _ => {
                todo!("Add error handling");
            }
        }
    }

    let body = parse_block(tokens);

    AstNode::FunctionDecl(FunctionDeclData {
        name,
        return_type,
        parameter,
        body,
    })
}

fn parse_block(tokens: &mut Peekable<std::slice::Iter<'_, Token>>) -> Box<AstNode> {
    tokens.next();
    let mut statements = Vec::new();

    loop {
        match tokens.peek() {
            Some(Token::RightBrace) => {
                tokens.next();
                break;
            }
            Some(Token::Return) => {
                statements.push(parse_return_statement(tokens));
            }
            _ => {
                todo!("Add error handling");
            }
        }
    }

    Box::new(AstNode::BlockStatement(statements))
}

fn parse_return_statement(tokens: &mut Peekable<std::slice::Iter<'_, Token>>) -> AstNode {
    tokens.next();

    let expr_node = match tokens.peek() {
        Some(Token::IntNumber(number)) => {
            tokens.next();
            let num = number.parse::<i16>().unwrap();
            AstNode::IntLiteralExpr(num)
        }
        Some(Token::Semicolon) => AstNode::IntLiteralExpr(0),
        _ => {
            todo!("Add error handling");
        }
    };

    match tokens.next() {
        Some(Token::Semicolon) => {}
        _ => {
            todo!("Add error handling");
        }
    }

    AstNode::ReturnStatement(Box::new(expr_node))
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
