use super::*;
use scanner::{Token, TokenKind};

macro_rules! t {
    ($kind:expr) => {
        Token::new(1, 1, $kind)
    };
}

macro_rules! toks {
    ( $source:literal ) => {{
        let mut scanner = Scanner::new($source);
        scanner.scan_source().expect("Scanner error in test setup")
    }};
    ( $( $kind:expr ),* $(,)? ) => {
        vec![ $( t!($kind) ),* ]
    };
}

macro_rules! int {
    ($val:expr) => {
        AstNode::IntLiteralExpr($val)
    };
}
macro_rules! var {
    ($name:expr) => {
        AstNode::VariableExpr($name)
    };
}

macro_rules! param {
    ($name:expr, $typ:expr) => {
        Parameter {
            name: $name,
            param_typ: $typ,
        }
    };
}

macro_rules! ret {
    ($expr:expr) => {
        AstNode::ReturnStatement(ReturnData {
            value: Some(Box::new($expr)),
            line: 1,
            column: 1,
        })
    };
    () => {
        AstNode::ReturnStatement(ReturnData {
            value: None,
            line: 1,
            column: 1,
        })
    };
}

macro_rules! block {
    ($($stmt:expr),* $(,)?) => {
        AstNode::BlockStatement(BlockStatementData {
            statements: vec![$($stmt),*],
            line: 1,
            column: 1,
        })
    };
}

macro_rules! bin_op {
    ($left:expr, $op:expr, $right:expr) => {
        AstNode::BinaryExpr(BinaryExpData {
            left: Box::new($left),
            operator: $op,
            right: Box::new($right),
            line: 1,
            column: 1,
        })
    };
}

macro_rules! assign {
    ($target:expr, $value:expr) => {
        AstNode::AssignmentStatement(AssignmentData {
            target: Box::new($target),
            value: Box::new($value),
            line: 1,
            column: 1,
        })
    };
}

macro_rules! assert_parse {
    ($tokens:expr, $expected_ast:expr) => {{
        let tokens = $tokens;
        let mut parser = Parser::new(&tokens);
        let ast = parser.parse().expect("Parsing failed unexpectedly");
        assert_eq!(ast, $expected_ast);
    }};
}

macro_rules! assert_parse_err {
    ($tokens:expr, $expected_pattern:pat) => {{
        let tokens = $tokens;
        let mut parser = Parser::new(&tokens);
        let result = parser.parse();
        assert!(
            matches!(result, Err($expected_pattern)),
            "Expected error matching stringified pattern, got {:?}",
            result
        );
    }};
}

#[test]
fn test_parse_simple_function() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("42"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(block!(ret!(int!(42)))),
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parse_simple_function_with_paramter() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::IntKeyword,
        TokenKind::Identifier("a"),
        TokenKind::Comma,
        TokenKind::IntKeyword,
        TokenKind::Identifier("b"),
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("42"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: vec![param!("a", Type::Int), param!("b", Type::Int)],
        body: Box::new(block!(ret!(int!(42)))),
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parse_empty_return_statement() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(block!(ret!())),
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parse_complex_function_expression() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("compute"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("10"),
        TokenKind::Plus,
        TokenKind::IntNumber("20"),
        TokenKind::Star,
        TokenKind::IntNumber("2"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "compute",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(block!(ret!(bin_op!(
            int!(10),
            BinaryOperator::Add,
            bin_op!(int!(20), BinaryOperator::Mul, int!(2))
        )))),
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parse_if_else_statement() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::If,
        TokenKind::LeftParen,
        TokenKind::IntNumber("5"),
        TokenKind::LessThan,
        TokenKind::IntNumber("10"),
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("1"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::Else,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("0"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::RightBrace,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(block!(AstNode::IfElseStatement(IfElseData {
            condition_expr: Box::new(bin_op!(int!(5), BinaryOperator::LessThan, int!(10))),
            if_branch: Box::new(block!(ret!(int!(1)))),
            else_branch: Some(Box::new(block!(ret!(int!(0))))),
            line: 1,
            column: 1,
        }))),
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parse_struct_declaration() {
    let tokens = toks!(
        TokenKind::StructKeyword,
        TokenKind::Identifier("Point"),
        TokenKind::LeftBrace,
        TokenKind::IntKeyword,
        TokenKind::Identifier("x"),
        TokenKind::Semicolon,
        TokenKind::IntKeyword,
        TokenKind::Identifier("y"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::Semicolon,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::StructDecl(StructDeclData {
        name: "Point",
        fields: vec![param!("x", Type::Int), param!("y", Type::Int)],
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parse_struct_type_decl() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::Identifier("Point"),
        TokenKind::Identifier("p"),
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("0"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: vec![param!("p", Type::Struct("Point"))],
        body: Box::new(block!(ret!(int!(0)))),
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parse_member_access() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Identifier("p"),
        TokenKind::Dot,
        TokenKind::Identifier("x"),
        TokenKind::Equal,
        TokenKind::IntNumber("5"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(block!(assign!(
            AstNode::MemberAccessExpr(MemberAccessData {
                object: Box::new(var!("p")),
                member: "x".to_string(),
                line: 1,
                column: 1,
            }),
            int!(5)
        ))),
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parse_while_statement() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::While,
        TokenKind::LeftParen,
        TokenKind::Identifier("i"),
        TokenKind::LessThan,
        TokenKind::IntNumber("5"),
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Identifier("i"),
        TokenKind::Equal,
        TokenKind::Identifier("i"),
        TokenKind::Plus,
        TokenKind::IntNumber("1"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::Return,
        TokenKind::Identifier("i"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(block!(
            AstNode::WhileStatement(WhileLoopData {
                condition_expr: Box::new(bin_op!(var!("i"), BinaryOperator::LessThan, int!(5))),
                body: Box::new(block!(assign!(
                    var!("i"),
                    bin_op!(var!("i"), BinaryOperator::Add, int!(1))
                ))),
                line: 1,
                column: 1,
            }),
            ret!(var!("i"))
        )),
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parse_print_statement() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::PrintKeyword,
        TokenKind::LeftParen,
        TokenKind::IntNumber("42"),
        TokenKind::RightParen,
        TokenKind::Semicolon,
        TokenKind::Return,
        TokenKind::IntNumber("0"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(block!(
            AstNode::PrintStatement(Box::new(int!(42))),
            ret!(int!(0))
        )),
        line: 1,
        column: 1,
    })]);

    assert_parse!(tokens, expected);
}

#[test]
fn test_parser_unexpected_eof() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::EoF
    );
    assert_parse_err!(tokens, ParserError::NotImplemented(_));
}

#[test]
fn test_parser_invalid_int_literal() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("999999999999999"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );
    assert_parse_err!(tokens, ParserError::InvalidIntLiteral { .. });
}

#[test]
fn test_parser_unexpected_global_token() {
    let tokens = toks!(TokenKind::Semicolon, TokenKind::EoF);
    assert_parse_err!(tokens, ParserError::UnexpectedToken { .. });
}

#[test]
fn test_parser_missing_left_brace() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::Return,
        TokenKind::IntNumber("0"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF
    );
    assert_parse_err!(tokens, ParserError::UnexpectedToken { .. });
}

#[test]
fn test_parser_not_implemented_global_var() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("global_var"),
        TokenKind::Semicolon,
        TokenKind::EoF
    );
    assert_parse_err!(tokens, ParserError::NotImplemented(_));
}

#[test]
fn test_parser_eof_inside_parameters() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::IntKeyword,
        TokenKind::EoF
    );
    let mut parser = Parser::new(&tokens);
    let result = parser.parse();

    match result {
        Err(ParserError::UnexpectedToken {
            expected,
            found,
            line,
            column,
        }) => {
            assert_eq!(expected, "Identifier");
            assert_eq!(found, "EoF");
            assert_eq!(line, 1);
            assert_eq!(column, 1);
        }
        other => panic!("Expected UnexpectedToken error, got {:?}", other),
    }
}

#[test]
fn test_parser_eof_inside_block() {
    let tokens = toks!(
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("0"),
        TokenKind::Semicolon,
        TokenKind::EoF
    );

    let mut parser = Parser::new(&tokens);
    let result = parser.parse();

    match result {
        Err(ParserError::UnexpectedToken {
            expected,
            found,
            line,
            column,
        }) => {
            assert_eq!(expected, "RightBrace");
            assert_eq!(found, "EoF");
            assert_eq!(line, 1);
            assert_eq!(column, 1);
        }
        other => panic!("Expected UnexpectedToken error, got {:?}", other),
    }
}

#[test]
fn test_parse_function_with_parameter() {
    let tokens = toks![
        TokenKind::IntKeyword,
        TokenKind::Identifier("add"),
        TokenKind::LeftParen,
        TokenKind::IntKeyword,
        TokenKind::Identifier("x"),
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("10"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();

    let expected = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "add",
        return_type: Type::Int,
        parameter: vec![Parameter {
            name: "x",
            param_typ: Type::Int,
        }],
        body: Box::new(block!(ret!(int!(10)))),
        line: 1,
        column: 1,
    })]);

    assert_eq!(ast, expected);
}

#[test]
fn test_parser_invalid_return_expression() {
    let tokens = toks![
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::LeftParen,
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF,
    ];

    let result = Parser::new(&tokens).parse();
    assert_eq!(
        result,
        Err(ParserError::UnexpectedToken {
            expected: "literal, identifier, or '('".to_string(),
            found: "Semicolon".to_string(),
            line: 1,
            column: 1,
        })
    );
}

#[test]
fn test_parse_if_without_else() {
    let tokens = toks![
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::If,
        TokenKind::LeftParen,
        TokenKind::Identifier("x"),
        TokenKind::DoubleEqual,
        TokenKind::IntNumber("0"),
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::IntNumber("42"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::RightBrace,
        TokenKind::EoF,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().unwrap();

    if let AstNode::Programm(nodes) = ast {
        if let AstNode::FunctionDecl(func) = &nodes[0] {
            if let AstNode::BlockStatement(stmts) = &*func.body {
                if let AstNode::IfElseStatement(if_data) = &stmts.statements[0] {
                    assert!(if_data.else_branch.is_none());
                    return;
                }
            }
        }
    }
    panic!("AST structure did not match expected IfElseStatement layout");
}

#[test]
fn test_parse_variable_decl_and_assignment() {
    let tokens = toks![
        TokenKind::LeftBrace,
        TokenKind::IntKeyword,
        TokenKind::Identifier("a"),
        TokenKind::Equal,
        TokenKind::IntNumber("5"),
        TokenKind::Semicolon,
        TokenKind::Identifier("a"),
        TokenKind::Equal,
        TokenKind::IntNumber("10"),
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF,
    ];

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse_block().unwrap();

    if let AstNode::BlockStatement(stmts) = *ast {
        assert_eq!(stmts.statements.len(), 2);
        assert!(matches!(stmts.statements[0], AstNode::VarDeclStatement(_)));
        assert!(matches!(
            stmts.statements[1],
            AstNode::AssignmentStatement(_)
        ));
    } else {
        panic!("Expected BlockStatement");
    }
}

#[test]
fn test_parse_array_declaration_and_index() {
    let tokens = toks![
        TokenKind::IntKeyword,
        TokenKind::Identifier("main"),
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::IntKeyword,
        TokenKind::LeftBracket,
        TokenKind::RightBracket,
        TokenKind::Identifier("arr"),
        TokenKind::Semicolon,
        TokenKind::Identifier("arr"),
        TokenKind::LeftBracket,
        TokenKind::IntNumber("0"),
        TokenKind::RightBracket,
        TokenKind::Equal,
        TokenKind::IntNumber("42"),
        TokenKind::Semicolon,
        TokenKind::Return,
        TokenKind::Identifier("arr"),
        TokenKind::LeftBracket,
        TokenKind::IntNumber("0"),
        TokenKind::RightBracket,
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::EoF,
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
