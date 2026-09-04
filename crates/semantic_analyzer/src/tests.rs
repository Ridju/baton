use super::*;

#[test]
fn test_semantic_valid_function() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(parser::FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::IntLiteralExpr(42))),
                line: 1,
                column: 1,
            })],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);
    let mut analyzer = Analyzer::new();
    assert!(analyzer.analyze(ast).is_ok());
}

#[test]
fn test_multiple_valid_functions() {
    let ast = AstNode::Programm(vec![
        AstNode::FunctionDecl(parser::FunctionDeclData {
            name: "foo",
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::IntLiteralExpr(10))),
                    line: 1,
                    column: 1,
                })],
                line: 1,
                column: 1,
            })),
            line: 1,
            column: 1,
        }),
        AstNode::FunctionDecl(parser::FunctionDeclData {
            name: "main",
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::IntLiteralExpr(42))),
                    line: 2,
                    column: 1,
                })],
                line: 2,
                column: 1,
            })),
            line: 2,
            column: 1,
        }),
    ]);
    let mut analyzer = Analyzer::new();
    assert!(analyzer.analyze(ast).is_ok());
}

#[test]
fn test_nested_blocks() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(parser::FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![AstNode::BlockStatement(parser::BlockStatementData {
                    statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                        value: Some(Box::new(AstNode::IntLiteralExpr(42))),
                        line: 1,
                        column: 1,
                    })],
                    line: 1,
                    column: 1,
                })],
                line: 1,
                column: 1,
            })],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);
    let mut analyzer = Analyzer::new();
    assert!(analyzer.analyze(ast).is_ok());
}

#[test]
fn test_semantic_error_redefinition() {
    let ast = AstNode::Programm(vec![
        AstNode::FunctionDecl(parser::FunctionDeclData {
            name: "main",
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::IntLiteralExpr(42))),
                    line: 1,
                    column: 1,
                })],
                line: 1,
                column: 1,
            })),
            line: 1,
            column: 1,
        }),
        AstNode::FunctionDecl(parser::FunctionDeclData {
            name: "main",
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::IntLiteralExpr(0))),
                    line: 2,
                    column: 1,
                })],
                line: 2,
                column: 1,
            })),
            line: 2,
            column: 1,
        }),
    ]);
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(matches!(result, Err(SemanticError::Redefinition { .. })));
}

#[test]
fn test_semantic_error_invalid_return_expression() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(parser::FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::BoolLiteralExpr(true))),
                line: 1,
                column: 1,
            })],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(matches!(result, Err(SemanticError::TypeMismatch { .. })));
}

#[test]
fn test_semantic_valid_if_else() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(parser::FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::VarDeclStatement(parser::VarDeclData {
                    name: "x",
                    var_typ: Type::Int,
                    initializer: Some(Box::new(AstNode::IntLiteralExpr(10))),
                    line: 1,
                    column: 1,
                }),
                AstNode::IfElseStatement(parser::IfElseData {
                    condition_expr: Box::new(AstNode::BinaryExpr(parser::BinaryExpData {
                        left: Box::new(AstNode::VariableExpr("x")),
                        operator: parser::BinaryOperator::GreaterThan,
                        right: Box::new(AstNode::IntLiteralExpr(5)),
                        line: 1,
                        column: 1,
                    })),
                    if_branch: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                        statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                            value: Some(Box::new(AstNode::IntLiteralExpr(1))),
                            line: 1,
                            column: 1,
                        })],
                        line: 1,
                        column: 1,
                    })),
                    else_branch: Some(Box::new(AstNode::BlockStatement(
                        parser::BlockStatementData {
                            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                                value: Some(Box::new(AstNode::IntLiteralExpr(0))),
                                line: 1,
                                column: 1,
                            })],
                            line: 1,
                            column: 1,
                        },
                    ))),
                    line: 1,
                    column: 1,
                }),
            ],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut analyzer = Analyzer::new();
    assert!(analyzer.analyze(ast).is_ok());
}

#[test]
fn test_semantic_error_undefined_variable_in_if_condition() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(parser::FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::IfElseStatement(parser::IfElseData {
                condition_expr: Box::new(AstNode::VariableExpr("undefined_var")),
                if_branch: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                    statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                        value: Some(Box::new(AstNode::IntLiteralExpr(0))),
                        line: 1,
                        column: 1,
                    })],
                    line: 1,
                    column: 1,
                })),
                else_branch: None,
                line: 1,
                column: 1,
            })],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(matches!(
        result,
        Err(SemanticError::UndefinedVariable { .. })
    ));
}

#[test]
fn test_funtion_call() {
    let ast = AstNode::Programm(vec![
        AstNode::FunctionDecl(parser::FunctionDeclData {
            name: "add",
            return_type: Type::Int,
            parameter: vec![
                parser::Parameter {
                    name: "a",
                    param_typ: Type::Int,
                },
                parser::Parameter {
                    name: "b",
                    param_typ: Type::Int,
                },
            ],
            body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::BinaryExpr(parser::BinaryExpData {
                        left: Box::new(AstNode::VariableExpr("a")),
                        operator: parser::BinaryOperator::Add,
                        right: Box::new(AstNode::VariableExpr("b")),
                        line: 1,
                        column: 1,
                    }))),
                    line: 1,
                    column: 1,
                })],
                line: 1,
                column: 1,
            })),
            line: 1,
            column: 1,
        }),
        AstNode::FunctionDecl(parser::FunctionDeclData {
            name: "main",
            return_type: Type::Int,
            parameter: vec![],
            body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::CallExpr(parser::CallData {
                        name: "add",
                        arguments: vec![AstNode::IntLiteralExpr(1), AstNode::IntLiteralExpr(3)],
                        line: 1,
                        column: 1,
                    }))),
                    line: 1,
                    column: 1,
                })],
                line: 1,
                column: 1,
            })),
            line: 1,
            column: 1,
        }),
    ]);

    let mut analyzer = Analyzer::new();
    assert!(analyzer.analyze(ast).is_ok());
}

#[test]
fn test_semantic_valid_bool_and_float() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(parser::FunctionDeclData {
        name: "main",
        return_type: Type::Bool,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::VarDeclStatement(parser::VarDeclData {
                    name: "isActive",
                    var_typ: Type::Bool,
                    initializer: Some(Box::new(AstNode::BoolLiteralExpr(true))),
                    line: 1,
                    column: 1,
                }),
                AstNode::VarDeclStatement(parser::VarDeclData {
                    name: "pi",
                    var_typ: Type::Float,
                    initializer: Some(Box::new(AstNode::FloatLiteralExpr(3.15))),
                    line: 1,
                    column: 1,
                }),
                AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::VariableExpr("isActive"))),
                    line: 1,
                    column: 1,
                }),
            ],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut analyzer = Analyzer::new();
    assert!(analyzer.analyze(ast).is_ok());
}

#[test]
fn test_semantic_error_assignment_type_mismatch() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(parser::FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::VarDeclStatement(parser::VarDeclData {
                    name: "x",
                    var_typ: Type::Int,
                    initializer: Some(Box::new(AstNode::IntLiteralExpr(10))),
                    line: 1,
                    column: 1,
                }),
                AstNode::AssignmentStatement(parser::AssignmentData {
                    target: Box::new(AstNode::VariableExpr("x")),
                    value: Box::new(AstNode::BoolLiteralExpr(false)),
                    line: 1,
                    column: 1,
                }),
                AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::VariableExpr("x"))),
                    line: 1,
                    column: 1,
                }),
            ],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(matches!(result, Err(SemanticError::TypeMismatch { .. })));
}

#[test]
fn test_semantic_error_binary_operand_mismatch() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(parser::FunctionDeclData {
        name: "main",
        return_type: Type::Float,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::BinaryExpr(parser::BinaryExpData {
                    left: Box::new(AstNode::IntLiteralExpr(1)),
                    operator: parser::BinaryOperator::Add,
                    right: Box::new(AstNode::FloatLiteralExpr(2.0)),
                    line: 1,
                    column: 1,
                }))),
                line: 1,
                column: 1,
            })],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(matches!(result, Err(SemanticError::TypeMismatch { .. })));
}

#[test]
fn test_semantic_valid_string_variable() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(parser::FunctionDeclData {
        name: "main",
        return_type: Type::String,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::VarDeclStatement(parser::VarDeclData {
                    name: "greeting",
                    var_typ: Type::String,
                    initializer: Some(Box::new(AstNode::StringLiteralExpr("Hello"))),
                    line: 1,
                    column: 1,
                }),
                AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::VariableExpr("greeting"))),
                    line: 1,
                    column: 1,
                }),
            ],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut analyzer = Analyzer::new();
    assert!(analyzer.analyze(ast).is_ok());
}
