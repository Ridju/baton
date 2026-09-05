use super::*;
use parser::{AstNode, BinaryExpData, FunctionDeclData, Type};

#[test]
fn test_generate_simple_main() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #256\n\tmov x0, #42\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n";

    assert_eq!(generator.buffer, expected_assembly);
}

#[test]
fn test_generate_zero_return() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::IntLiteralExpr(0))),
                line: 1,
                column: 1,
            })],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();
    let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #256\n\tmov x0, #0\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n";

    assert_eq!(generator.buffer, expected_assembly);
}

#[test]
fn test_generate_multiple_functions() {
    let ast = AstNode::Programm(vec![
        AstNode::FunctionDecl(FunctionDeclData {
            name: "main",
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::IntLiteralExpr(0))),
                    line: 1,
                    column: 1,
                })],
                line: 1,
                column: 1,
            })),
            line: 1,
            column: 1,
        }),
        AstNode::FunctionDecl(FunctionDeclData {
            name: "helper_func",
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::IntLiteralExpr(100))),
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #256\n\tmov x0, #0\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n_helper_func:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #256\n\tmov x0, #100\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n";

    assert_eq!(generator.buffer, expected_assembly);
}

#[test]
fn test_generate_block_with_multiple_statements() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::IntLiteralExpr(5),
                AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::IntLiteralExpr(10))),
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #256\n\tmov x0, #5\n\tmov x0, #10\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n";

    assert_eq!(generator.buffer, expected_assembly);
}

#[test]
fn test_codegen_binary_expression() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::BinaryExpr(BinaryExpData {
                    left: Box::new(AstNode::IntLiteralExpr(200)),
                    right: Box::new(AstNode::IntLiteralExpr(20)),
                    operator: BinaryOperator::Add,
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

    let mut codegen = Generator::new();
    codegen.generate(ast).unwrap();

    assert!(codegen.buffer.contains(".global _main") || codegen.buffer.contains(".global main"));
    assert!(codegen.buffer.contains("add"));
    assert!(codegen.buffer.contains("str"));
    assert!(codegen.buffer.contains("ldr"));
    assert!(codegen.buffer.contains("ret"));
}

#[test]
fn test_codegen_if_else_statement() {
    use parser::{BinaryExpData, BinaryOperator, IfElseData};

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::IfElseStatement(IfElseData {
                condition_expr: Box::new(AstNode::BinaryExpr(BinaryExpData {
                    left: Box::new(AstNode::IntLiteralExpr(10)),
                    right: Box::new(AstNode::IntLiteralExpr(5)),
                    operator: BinaryOperator::GreaterThan,
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
            })],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;

    assert!(assembly.contains("cmp"));
    assert!(assembly.contains("cset"));
    assert!(assembly.contains("gt"));
    assert!(assembly.contains("cbz"));
    assert!(assembly.contains(".L_else_0"));
    assert!(assembly.contains(".L_end_0"));
}

#[test]
fn test_generate_function_with_parameters() {
    use parser::{BinaryExpData, BinaryOperator, Parameter};

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "add",
        return_type: Type::Int,
        parameter: vec![
            Parameter {
                name: "a",
                param_typ: Type::Int,
            },
            Parameter {
                name: "b",
                param_typ: Type::Int,
            },
        ],
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::BinaryExpr(BinaryExpData {
                    left: Box::new(AstNode::VariableExpr("a")),
                    right: Box::new(AstNode::VariableExpr("b")),
                    operator: BinaryOperator::Add,
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;
    assert!(assembly.contains("_add:"));

    assert!(assembly.contains("\tstr x0, [x29, #-8]"));
    assert!(assembly.contains("\tstr x1, [x29, #-16]"));

    assert!(assembly.contains("\tldr x0, [x29, #-8]"));
    assert!(assembly.contains("\tldr x0, [x29, #-16]"));

    assert!(assembly.contains("\tadd x0, x1, x0"));
}

#[test]
fn test_generate_function_call() {
    use parser::CallData;

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: vec![],
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::CallExpr(CallData {
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
    })]);

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;

    assert!(assembly.contains("\tstr x0, [sp, #-16]!"));
    assert!(assembly.contains("\tldr x0, [sp], #16"));
    assert!(assembly.contains("\tldr x1, [sp], #16"));
    assert!(assembly.contains("\tbl _add"));
}

#[test]
fn test_codegen_bool_literal() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Bool,
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;
    assert!(assembly.contains("mov x0, #1"));
}

#[test]
fn test_codegen_float_literal() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Float,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::FloatLiteralExpr(3.15))),
                line: 1,
                column: 1,
            })],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;
    assert!(assembly.contains("__TEXT,__const"));
    assert!(assembly.contains("L_float_0"));
    assert!(assembly.contains("adrp x16, L_float_0@PAGE"));
    assert!(assembly.contains("ldr d0, [x16, L_float_0@PAGEOFF]"));
}

#[test]
fn test_codegen_string_literal() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::String,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::StringLiteralExpr("Hello World"))),
                line: 1,
                column: 1,
            })],
            line: 1,
            column: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;
    assert!(assembly.contains("__TEXT,__cstring"));
    assert!(assembly.contains(".asciz \"Hello World\""));
    assert!(assembly.contains("adrp x0, L_str_0@PAGE"));
    assert!(assembly.contains("add x0, x0, L_str_0@PAGEOFF"));
}

#[test]
fn test_codegen_float_binary_expression() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Float,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![AstNode::ReturnStatement(parser::ReturnData {
                value: Some(Box::new(AstNode::BinaryExpr(BinaryExpData {
                    left: Box::new(AstNode::FloatLiteralExpr(1.5)),
                    right: Box::new(AstNode::FloatLiteralExpr(2.5)),
                    operator: BinaryOperator::Add,
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;
    assert!(assembly.contains("adrp x16, L_float_0@PAGE"));
    assert!(assembly.contains("str d0, [sp, #-16]!"));
    assert!(assembly.contains("ldr d1, [sp], #16"));
    assert!(assembly.contains("fadd d0, d1, d0"));
}

#[test]
fn test_codegen_struct_decl_and_member_access() {
    use parser::{MemberAccessData, Parameter, StructDeclData};

    let ast = AstNode::Programm(vec![
        AstNode::StructDecl(StructDeclData {
            name: "Point",
            fields: vec![
                Parameter {
                    name: "x",
                    param_typ: Type::Int,
                },
                Parameter {
                    name: "y",
                    param_typ: Type::Int,
                },
            ],
            line: 1,
            column: 1,
        }),
        AstNode::FunctionDecl(FunctionDeclData {
            name: "main",
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                statements: vec![
                    AstNode::VarDeclStatement(parser::VarDeclData {
                        name: "p",
                        var_typ: Type::Struct("Point"),
                        initializer: None,
                        line: 1,
                        column: 1,
                    }),
                    AstNode::AssignmentStatement(parser::AssignmentData {
                        target: Box::new(AstNode::MemberAccessExpr(MemberAccessData {
                            object: Box::new(AstNode::VariableExpr("p")),
                            member: "y".to_string(),
                            line: 1,
                            column: 1,
                        })),
                        value: Box::new(AstNode::IntLiteralExpr(10)),
                        line: 1,
                        column: 1,
                    }),
                    AstNode::ReturnStatement(parser::ReturnData {
                        value: Some(Box::new(AstNode::MemberAccessExpr(MemberAccessData {
                            object: Box::new(AstNode::VariableExpr("p")),
                            member: "y".to_string(),
                            line: 1,
                            column: 1,
                        }))),
                        line: 1,
                        column: 1,
                    }),
                ],
                line: 1,
                column: 1,
            })),
            line: 1,
            column: 1,
        }),
    ]);

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();
    let assembly = generator.buffer;

    assert!(assembly.contains("_main:"));
    assert!(assembly.contains("add x0, x29, #"));
    assert!(assembly.contains("str x0, [x1, #8]"));
    assert!(assembly.contains("ret"));
}

#[test]
fn test_codegen_variable_declaration_with_initializer() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::VarDeclStatement(parser::VarDeclData {
                    name: "a",
                    var_typ: Type::Int,
                    initializer: Some(Box::new(AstNode::IntLiteralExpr(99))),
                    line: 1,
                    column: 1,
                }),
                AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::VariableExpr("a"))),
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();
    let assembly = generator.buffer;

    assert!(assembly.contains("mov x0, #99"));
    assert!(assembly.contains("str x0, [x29, #"));
    assert!(assembly.contains("ldr x0, [x29, #"));
}

#[test]
fn test_codegen_array_index_expression() {
    use parser::{ArrayIndexData, VarDeclData};

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::VarDeclStatement(VarDeclData {
                    name: "arr",
                    var_typ: Type::Array(Box::new(Type::Int)),
                    initializer: None,
                    line: 1,
                    column: 1,
                }),
                AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::ArrayIndexExpr(ArrayIndexData {
                        array: Box::new(AstNode::VariableExpr("arr")),
                        index: Box::new(AstNode::IntLiteralExpr(2)),
                        line: 1,
                        column: 1,
                    }))),
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;
    assert!(assembly.contains("lsl x0, x0, #3") || assembly.contains("mul x0, x0, x2"));
    assert!(assembly.contains("add x0, x1, x0"));
    assert!(assembly.contains("ldr x0, [x0]"));
}

#[test]
fn test_codegen_while_statement() {
    use parser::{AssignmentData, BinaryExpData, BinaryOperator, WhileLoopData};

    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::VarDeclStatement(parser::VarDeclData {
                    name: "i",
                    var_typ: Type::Int,
                    initializer: Some(Box::new(AstNode::IntLiteralExpr(0))),
                    line: 1,
                    column: 1,
                }),
                AstNode::WhileStatement(WhileLoopData {
                    condition_expr: Box::new(AstNode::BinaryExpr(BinaryExpData {
                        left: Box::new(AstNode::VariableExpr("i")),
                        right: Box::new(AstNode::IntLiteralExpr(5)),
                        operator: BinaryOperator::LessThan,
                        line: 1,
                        column: 1,
                    })),
                    body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
                        statements: vec![AstNode::AssignmentStatement(AssignmentData {
                            target: Box::new(AstNode::VariableExpr("i")),
                            value: Box::new(AstNode::BinaryExpr(BinaryExpData {
                                left: Box::new(AstNode::VariableExpr("i")),
                                right: Box::new(AstNode::IntLiteralExpr(1)),
                                operator: BinaryOperator::Add,
                                line: 1,
                                column: 1,
                            })),
                            line: 1,
                            column: 1,
                        })],
                        line: 1,
                        column: 1,
                    })),
                    line: 1,
                    column: 1,
                }),
                AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::VariableExpr("i"))),
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;

    assert!(assembly.contains(".L_while_start_0:"));
    assert!(assembly.contains(".L_while_end_0:"));
    assert!(assembly.contains("\tcbz x0, .L_while_end_0"));
    assert!(assembly.contains("\tb .L_while_start_0"));
    assert!(assembly.contains("cmp"));
    assert!(assembly.contains("cset"));
    assert!(assembly.contains("lt"));
}

#[test]
fn test_codegen_print_statement() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::PrintStatement(Box::new(AstNode::IntLiteralExpr(42))),
                AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::IntLiteralExpr(0))),
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

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;

    assert!(assembly.contains("__TEXT,__cstring"));
    assert!(assembly.contains(".asciz \"%d\\n\""));
    assert!(assembly.contains("sub sp, sp, #16"));
    assert!(assembly.contains("str x0, [sp]"));
    assert!(assembly.contains("bl _printf"));
    assert!(assembly.contains("add sp, sp, #16"));
}

#[test]
fn test_codegen_print_string() {
    let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
        name: "main",
        return_type: Type::Int,
        parameter: Vec::new(),
        body: Box::new(AstNode::BlockStatement(parser::BlockStatementData {
            statements: vec![
                AstNode::PrintStatement(Box::new(AstNode::StringLiteralExpr("Hello Print"))),
                AstNode::ReturnStatement(parser::ReturnData {
                    value: Some(Box::new(AstNode::IntLiteralExpr(0))),
                    line: 1,
                    column: 1,
                }),
            ],
            column: 1,
            line: 1,
        })),
        line: 1,
        column: 1,
    })]);

    let mut generator = Generator::new();
    generator.generate(ast).unwrap();

    let assembly = generator.buffer;

    assert!(assembly.contains(".asciz \"Hello Print\""));
    assert!(assembly.contains(".asciz \"%s\\n\""));
    assert!(assembly.contains("bl _printf"));
}
