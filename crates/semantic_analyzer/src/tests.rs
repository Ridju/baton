use super::*;

#[test]
fn test_semantic_valid_function() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
            Box::new(AstNode::IntLiteralExpr(42)),
        )])),
    })]);
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);

    assert!(result.is_ok());
}

#[test]
fn test_multiple_valid_functions() {
    let ast = AstNode::Programm(vec![
        AstNode::FunctionDecl(FunctionDeclData {
            name: "foo".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(10)),
            )])),
        }),
        AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(42)),
            )])),
        }),
    ]);
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);

    assert!(result.is_ok());
}

#[test]
fn test_nested_blocks() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![AstNode::BlockStatement(
            vec![AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(42)),
            )])],
        )])),
    })]);
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);

    assert!(result.is_ok());
}
#[test]
fn test_semantic_error_redefinition() {
    let ast = AstNode::Programm(vec![
        AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(42)),
            )])),
        }),
        AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(0)),
            )])),
        }),
    ]);
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);

    assert!(result.is_err());
    match result {
        Err(SemanticError::Redefinition(_)) => {}
        _ => panic!("Expected Redefinition error, got something else"),
    }
}
#[test]
fn test_semantic_error_invalid_return_expression() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
            Box::new(AstNode::BlockStatement(vec![])),
        )])),
    })]);
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);

    assert!(result.is_err());
    match result {
        Err(SemanticError::InvalidReturnType(_)) => {}
        _ => panic!("Expected InvalidReturnType error"),
    }
}

#[test]
fn test_semantic_valid_return_type() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
            Box::new(AstNode::IntLiteralExpr(42)),
        )])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);

    assert!(result.is_ok());
}

#[test]
fn test_semantic_valid_if_else() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::VarDeclStatement(parser::VarDeclData {
                name: "x".to_string(),
                var_typ: Type::Int,
                initializer: Some(Box::new(AstNode::IntLiteralExpr(10))),
            }),
            AstNode::IfElseStatement(parser::IfElseData {
                condition_expr: Box::new(AstNode::BinaryExpr(parser::BinaryExpData {
                    left: Box::new(AstNode::VariableExpr("x".to_string())),
                    right: Box::new(AstNode::IntLiteralExpr(5)),
                    operator: parser::BinaryOperator::GreaterThan,
                })),
                if_branch: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                    Box::new(AstNode::IntLiteralExpr(1)),
                )])),
                else_branch: Some(Box::new(AstNode::BlockStatement(vec![
                    AstNode::ReturnStatement(Box::new(AstNode::IntLiteralExpr(0))),
                ]))),
            }),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_error_undefined_variable_in_if_condition() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![AstNode::IfElseStatement(
            parser::IfElseData {
                condition_expr: Box::new(AstNode::VariableExpr("undefined_var".to_string())),
                if_branch: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                    Box::new(AstNode::IntLiteralExpr(0)),
                )])),
                else_branch: None,
            },
        )])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(matches!(result, Err(SemanticError::UndefinedVariable(_))));
}

#[test]
fn test_funtion_call() {
    use parser::{BinaryExpData, BinaryOperator, CallData, Parameter};

    let ast = AstNode::Programm(vec![
        AstNode::FunctionDecl(FunctionDeclData {
            name: "add".to_string(),
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
                Box::new(AstNode::BinaryExpr(BinaryExpData {
                    left: Box::new(AstNode::VariableExpr("a".to_string())),
                    right: Box::new(AstNode::VariableExpr("b".to_string())),
                    operator: BinaryOperator::Add,
                })),
            )])),
        }),
        AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: vec![],
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::CallExpr(CallData {
                    name: "add".to_string(),
                    arguments: vec![AstNode::IntLiteralExpr(1), AstNode::IntLiteralExpr(3)],
                })),
            )])),
        }),
    ]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Der Semantic Analyzer sollte den Funktionsaufruf als gültig erkennen."
    );
}

#[test]
fn test_semantic_valid_bool_and_float() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Bool,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::VarDeclStatement(parser::VarDeclData {
                name: "isActive".to_string(),
                var_typ: Type::Bool,
                initializer: Some(Box::new(AstNode::BoolLiteralExpr(true))),
            }),
            AstNode::VarDeclStatement(parser::VarDeclData {
                name: "pi".to_string(),
                var_typ: Type::Float,
                initializer: Some(Box::new(AstNode::FloatLiteralExpr(3.14))),
            }),
            AstNode::ReturnStatement(Box::new(AstNode::VariableExpr("isActive".to_string()))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Der Analyzer sollte gültige Bool- und Float-Deklarationen akzeptieren."
    );
}

#[test]
fn test_semantic_error_assignment_type_mismatch() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::VarDeclStatement(parser::VarDeclData {
                name: "x".to_string(),
                var_typ: Type::Int,
                initializer: Some(Box::new(AstNode::IntLiteralExpr(10))),
            }),
            AstNode::AssignmentStatement(parser::AssignmentData {
                target: Box::new(AstNode::VariableExpr("x".to_string())),
                value: Box::new(AstNode::BoolLiteralExpr(false)),
            }),
            AstNode::ReturnStatement(Box::new(AstNode::VariableExpr("x".to_string()))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        matches!(result, Err(SemanticError::NotMatchingReturnType(_))),
        "Sollte einen Typ-Fehler bei Zuweisung werfen."
    );
}

#[test]
fn test_semantic_error_binary_operand_mismatch() {
    use parser::{BinaryExpData, BinaryOperator};

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Float,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
            Box::new(AstNode::BinaryExpr(BinaryExpData {
                left: Box::new(AstNode::IntLiteralExpr(1)),
                right: Box::new(AstNode::FloatLiteralExpr(2.0)),
                operator: BinaryOperator::Add,
            })),
        )])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        matches!(result, Err(SemanticError::NotMatchingReturnType(_))),
        "Sollte Operandentyp-Inkompatibilität (Int vs. Float) erkennen."
    );
}

#[test]
fn test_semantic_valid_string_variable() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::String,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::VarDeclStatement(parser::VarDeclData {
                name: "greeting".to_string(),
                var_typ: Type::String,
                initializer: Some(Box::new(AstNode::StringLiteralExpr("Hello".to_string()))),
            }),
            AstNode::ReturnStatement(Box::new(AstNode::VariableExpr("greeting".to_string()))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "String-Variablen und deren Rückgabe sollten valide sein."
    );
}
#[test]
fn test_semantic_valid_struct_and_member_access() {
    use parser::{MemberAccessData, Parameter, StructDeclData};

    let ast = AstNode::Programm(vec![
        AstNode::StructDecl(StructDeclData {
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
        }),
        AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![
                AstNode::VarDeclStatement(parser::VarDeclData {
                    name: "p".to_string(),
                    var_typ: Type::Struct("Point".to_string()),
                    initializer: None,
                }),
                AstNode::AssignmentStatement(parser::AssignmentData {
                    target: Box::new(AstNode::MemberAccessExpr(MemberAccessData {
                        object: Box::new(AstNode::VariableExpr("p".to_string())),
                        member: "x".to_string(),
                    })),
                    value: Box::new(AstNode::IntLiteralExpr(10)),
                }),
                AstNode::ReturnStatement(Box::new(AstNode::MemberAccessExpr(MemberAccessData {
                    object: Box::new(AstNode::VariableExpr("p".to_string())),
                    member: "x".to_string(),
                }))),
            ])),
        }),
    ]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Struct-Deklaration, Member-Zuweisung und Member-Access sollten valide sein."
    );
}

#[test]
fn test_semantic_error_unknown_struct_field() {
    use parser::{MemberAccessData, Parameter, StructDeclData};

    let ast = AstNode::Programm(vec![
        AstNode::StructDecl(StructDeclData {
            name: "Point".to_string(),
            fields: vec![Parameter {
                name: "x".to_string(),
                param_typ: Type::Int,
            }],
        }),
        AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![
                AstNode::VarDeclStatement(parser::VarDeclData {
                    name: "p".to_string(),
                    var_typ: Type::Struct("Point".to_string()),
                    initializer: None,
                }),
                AstNode::ReturnStatement(Box::new(AstNode::MemberAccessExpr(MemberAccessData {
                    object: Box::new(AstNode::VariableExpr("p".to_string())),
                    member: "y".to_string(),
                }))),
            ])),
        }),
    ]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        matches!(result, Err(SemanticError::UndefinedVariable(_))),
        "Sollte einen Fehler werfen, wenn auf ein nicht existierendes Struct-Feld zugegriffen wird."
    );
}

#[test]
fn test_semantic_error_member_access_on_primitive() {
    use parser::MemberAccessData;

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::VarDeclStatement(parser::VarDeclData {
                name: "x".to_string(),
                var_typ: Type::Int,
                initializer: Some(Box::new(AstNode::IntLiteralExpr(5))),
            }),
            AstNode::ReturnStatement(Box::new(AstNode::MemberAccessExpr(MemberAccessData {
                object: Box::new(AstNode::VariableExpr("x".to_string())),
                member: "field".to_string(),
            }))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        matches!(result, Err(SemanticError::NotMatchingReturnType(_))),
        "Sollte einen Fehler werfen, wenn der Punkt-Operator auf einen primitiven Typ angewendet wird."
    );
}

#[test]
fn test_semantic_error_struct_redefinition() {
    use parser::{Parameter, StructDeclData};

    let ast = AstNode::Programm(vec![
        AstNode::StructDecl(StructDeclData {
            name: "Point".to_string(),
            fields: vec![Parameter {
                name: "x".to_string(),
                param_typ: Type::Int,
            }],
        }),
        AstNode::StructDecl(StructDeclData {
            name: "Point".to_string(),
            fields: vec![Parameter {
                name: "y".to_string(),
                param_typ: Type::Int,
            }],
        }),
    ]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        matches!(result, Err(SemanticError::Redefinition(_))),
        "Sollte einen Redefinitionsfehler bei doppelten Struct-Namen werfen."
    );
}
#[test]
fn test_semantic_valid_array_indexing() {
    use parser::{ArrayIndexData, VarDeclData};

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::VarDeclStatement(VarDeclData {
                name: "arr".to_string(),
                var_typ: Type::Array(Box::new(Type::Int)),
                initializer: None,
            }),
            AstNode::ReturnStatement(Box::new(AstNode::ArrayIndexExpr(ArrayIndexData {
                array: Box::new(AstNode::VariableExpr("arr".to_string())),
                index: Box::new(AstNode::IntLiteralExpr(0)),
            }))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Gültiger Array-Index-Zugriff sollte erfolgreich analysiert werden."
    );
}

#[test]
fn test_semantic_error_invalid_array_index_type() {
    use parser::{ArrayIndexData, VarDeclData};

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::VarDeclStatement(VarDeclData {
                name: "arr".to_string(),
                var_typ: Type::Array(Box::new(Type::Int)),
                initializer: None,
            }),
            AstNode::ReturnStatement(Box::new(AstNode::ArrayIndexExpr(ArrayIndexData {
                array: Box::new(AstNode::VariableExpr("arr".to_string())),
                index: Box::new(AstNode::StringLiteralExpr("invalid_index".to_string())),
            }))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        matches!(result, Err(SemanticError::NotMatchingReturnType(_))),
        "Sollte einen Fehler werfen, wenn der Array-Index kein Integer ist."
    );
}

#[test]
fn test_semantic_error_variable_redefinition() {
    use parser::VarDeclData;

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::VarDeclStatement(VarDeclData {
                name: "x".to_string(),
                var_typ: Type::Int,
                initializer: Some(Box::new(AstNode::IntLiteralExpr(1))),
            }),
            AstNode::VarDeclStatement(VarDeclData {
                name: "x".to_string(),
                var_typ: Type::Int,
                initializer: Some(Box::new(AstNode::IntLiteralExpr(2))),
            }),
            AstNode::ReturnStatement(Box::new(AstNode::VariableExpr("x".to_string()))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        matches!(result, Err(SemanticError::Redefinition(_))),
        "Sollte eine Redefinition von Variablen im selben Scope erkennen."
    );
}
#[test]
fn test_semantic_valid_while_loop() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::VarDeclStatement(parser::VarDeclData {
                name: "i".to_string(),
                var_typ: Type::Int,
                initializer: Some(Box::new(AstNode::IntLiteralExpr(0))),
            }),
            AstNode::WhileStatement(parser::WhileLoopData {
                condition_expr: Box::new(AstNode::BinaryExpr(parser::BinaryExpData {
                    left: Box::new(AstNode::VariableExpr("i".to_string())),
                    right: Box::new(AstNode::IntLiteralExpr(5)),
                    operator: parser::BinaryOperator::LessThan,
                })),
                body: Box::new(AstNode::BlockStatement(vec![AstNode::AssignmentStatement(
                    parser::AssignmentData {
                        target: Box::new(AstNode::VariableExpr("i".to_string())),
                        value: Box::new(AstNode::BinaryExpr(parser::BinaryExpData {
                            left: Box::new(AstNode::VariableExpr("i".to_string())),
                            right: Box::new(AstNode::IntLiteralExpr(1)),
                            operator: parser::BinaryOperator::Add,
                        })),
                    },
                )])),
            }),
            AstNode::ReturnStatement(Box::new(AstNode::VariableExpr("i".to_string()))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Der Analyzer sollte eine valide While-Schleife mit boolscher Bedingung akzeptieren."
    );
}

#[test]
fn test_semantic_error_while_non_bool_condition() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::WhileStatement(parser::WhileLoopData {
                condition_expr: Box::new(AstNode::IntLiteralExpr(1)),
                body: Box::new(AstNode::BlockStatement(vec![])),
            }),
            AstNode::ReturnStatement(Box::new(AstNode::IntLiteralExpr(0))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        matches!(result, Err(SemanticError::NotMatchingReturnType(_))),
        "Sollte einen Fehler werfen, wenn die While-Bedingung kein Bool ist."
    );
}
#[test]
fn test_analyze_print_statement_valid() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: vec![],
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::PrintStatement(Box::new(AstNode::IntLiteralExpr(42))),
            AstNode::PrintStatement(Box::new(AstNode::StringLiteralExpr("Hallo".to_string()))),
            AstNode::PrintStatement(Box::new(AstNode::BoolLiteralExpr(true))),
            AstNode::ReturnStatement(Box::new(AstNode::IntLiteralExpr(0))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);

    assert!(
        result.is_ok(),
        "Semantic analysis failed for valid print statements: {:?}",
        result
    );
}

#[test]
fn test_analyze_print_undefined_variable() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main".to_string(),
        return_type: Type::Int,
        parameter: vec![],
        body: Box::new(AstNode::BlockStatement(vec![
            AstNode::PrintStatement(Box::new(AstNode::VariableExpr("x".to_string()))),
            AstNode::ReturnStatement(Box::new(AstNode::IntLiteralExpr(0))),
        ])),
    })]);

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(ast);

    assert!(matches!(result, Err(SemanticError::UndefinedVariable(_))));
}
