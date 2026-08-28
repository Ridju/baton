use super::*;
use scanner::Token;

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
            expected: "type keyword (int, bool, float, string) or struct".to_string(),
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

#[test]
fn test_parse_struct_declaration() {
    let tokens = vec![
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
        Token::Semicolon,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();

    let expected = AstNode::Programm(vec![AstNode::StructDecl(StructDeclData {
        name: "Point".to_string(),
        fields: vec![
            Parameter {
                name: "x".to_string(),
                param_typ: Type::Int,
            },
            Parameter {
                name: "y".to_string(),
                param_typ: Type::Int,
            },
        ],
    })]);

    assert_eq!(ast, expected);
}

#[test]
fn test_parse_variable_decl_and_assignment() {
    let tokens = vec![
        Token::LeftBrace,
        Token::IntKeyword,
        Token::Identifier("a".to_string()),
        Token::Equal,
        Token::IntNumber("5".to_string()),
        Token::Semicolon,
        Token::Identifier("a".to_string()),
        Token::Equal,
        Token::IntNumber("10".to_string()),
        Token::Semicolon,
        Token::RightBrace,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse_block().unwrap();

    if let AstNode::BlockStatement(stmts) = *ast {
        assert_eq!(stmts.len(), 2);
        assert!(matches!(stmts[0], AstNode::VarDeclStatement(_)));
        assert!(matches!(stmts[1], AstNode::AssignmentStatement(_)));
    } else {
        panic!("Expected BlockStatement");
    }
}

#[test]
fn test_parse_member_access() {
    let tokens = vec![
        Token::Identifier("p".to_string()),
        Token::Dot,
        Token::Identifier("x".to_string()),
        Token::Equal,
        Token::IntNumber("5".to_string()),
        Token::Semicolon,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse_assignment().unwrap();

    if let AstNode::AssignmentStatement(data) = ast {
        assert!(matches!(*data.target, AstNode::MemberAccessExpr(_)));
        if let AstNode::MemberAccessExpr(member) = *data.target {
            assert_eq!(member.member, "x");
            assert_eq!(
                member.object,
                Box::new(AstNode::VariableExpr("p".to_string()))
            );
        }
    } else {
        panic!("Expected AssignmentStatement");
    }
}

#[test]
fn test_parse_struct_type_decl() {
    let tokens = vec![
        Token::IntKeyword,
        Token::Identifier("main".to_string()),
        Token::LeftParen,
        Token::Identifier("Point".to_string()),
        Token::Identifier("p".to_string()),
        Token::RightParen,
        Token::LeftBrace,
        Token::Return,
        Token::IntNumber("0".to_string()),
        Token::Semicolon,
        Token::RightBrace,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();

    if let AstNode::Programm(mut nodes) = ast {
        if let AstNode::FunctionDecl(f) = nodes.remove(0) {
            assert_eq!(f.parameter[0].param_typ, Type::Struct("Point".to_string()));
        }
    }
}
#[test]
fn test_parse_array_declaration_and_index() {
    let tokens = vec![
        Token::IntKeyword,
        Token::Identifier("main".to_string()),
        Token::LeftParen,
        Token::RightParen,
        Token::LeftBrace,
        Token::IntKeyword,
        Token::LeftBracket,
        Token::RightBracket,
        Token::Identifier("arr".to_string()),
        Token::Semicolon,
        Token::Identifier("arr".to_string()),
        Token::LeftBracket,
        Token::IntNumber("0".to_string()),
        Token::RightBracket,
        Token::Equal,
        Token::IntNumber("42".to_string()),
        Token::Semicolon,
        Token::Return,
        Token::Identifier("arr".to_string()),
        Token::LeftBracket,
        Token::IntNumber("0".to_string()),
        Token::RightBracket,
        Token::Semicolon,
        Token::RightBrace,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();

    match ast {
        AstNode::Programm(nodes) => {
            assert_eq!(nodes.len(), 1);
        }
        _ => panic!("Expected Programm AST node"),
    }
}

#[test]
fn test_parse_while_statement() {
    let tokens = vec![
        Token::IntKeyword,
        Token::Identifier("main".to_string()),
        Token::LeftParen,
        Token::RightParen,
        Token::LeftBrace,
        Token::While,
        Token::LeftParen,
        Token::Identifier("i".to_string()),
        Token::LessThan,
        Token::IntNumber("5".to_string()),
        Token::RightParen,
        Token::LeftBrace,
        Token::Identifier("i".to_string()),
        Token::Equal,
        Token::Identifier("i".to_string()),
        Token::Plus,
        Token::IntNumber("1".to_string()),
        Token::Semicolon,
        Token::RightBrace,
        Token::Return,
        Token::Identifier("i".to_string()),
        Token::Semicolon,
        Token::RightBrace,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::WhileStatement(WhileLoopData {
                condition_expr: Box::new(AstNode::BinaryExpr(BinaryExpData {
                    left: Box::new(AstNode::VariableExpr("i".to_string())),
                    right: Box::new(AstNode::IntLiteralExpr(5)),
                    operator: BinaryOperator::LessThan,
                })),
                body: Box::new(AstNode::BlockStatement(vec![AstNode::AssignmentStatement(
                    AssignmentData {
                        target: Box::new(AstNode::VariableExpr("i".to_string())),
                        value: Box::new(AstNode::BinaryExpr(BinaryExpData {
                            left: Box::new(AstNode::VariableExpr("i".to_string())),
                            right: Box::new(AstNode::IntLiteralExpr(1)),
                            operator: BinaryOperator::Add,
                        })),
                    },
                )])),
            }),
            AstNode::ReturnStatement(Box::new(AstNode::VariableExpr("i".to_string()))),
        ])),
    })]);

    assert_eq!(ast, expected);
}

#[test]
fn test_parse_print_statement() {
    let tokens = vec![
        Token::IntKeyword,
        Token::Identifier("main".to_string()),
        Token::LeftParen,
        Token::RightParen,
        Token::LeftBrace,
        Token::PrintKeyword,
        Token::LeftParen,
        Token::IntNumber("42".to_string()),
        Token::RightParen,
        Token::Semicolon,
        Token::Return,
        Token::IntNumber("0".to_string()),
        Token::Semicolon,
        Token::RightBrace,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();

    if let AstNode::Programm(nodes) = ast {
        assert_eq!(nodes.len(), 1);
        if let AstNode::FunctionDecl(func) = &nodes[0] {
            assert_eq!(func.name, "main");

            if let AstNode::BlockStatement(statements) = &*func.body {
                assert_eq!(statements.len(), 2);
                assert_eq!(
                    statements[0],
                    AstNode::PrintStatement(Box::new(AstNode::IntLiteralExpr(42)))
                );
            } else {
                panic!("Expected block statement in function body");
            }
        } else {
            panic!("Expected function declaration");
        }
    } else {
        panic!("Expected program root");
    }
}
