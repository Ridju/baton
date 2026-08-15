use crate::parser::{AstNode, FunctionDeclData, Type};
use std::collections::HashMap;

#[derive(Debug)]
pub enum SemanticError {
    Redefinition(String),
    InvalidReturnType(String),
    UndefinedVariable(String),
    NotMatchingReturnType(String),
    NotImplemented(String),
}

enum ElementKind {
    Function,
    Variable,
    Parameter,
}

struct MetaData {
    typ: Type,
    kind: ElementKind,
    parameters: Option<Vec<Type>>,
}

pub struct Analyzer {
    scope_stack: Vec<HashMap<String, MetaData>>,
    current_return_type: Option<Type>,
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            scope_stack: vec![HashMap::new()],
            current_return_type: None,
        }
    }

    pub fn analyze(&mut self, ast: AstNode) -> Result<(), SemanticError> {
        match ast {
            AstNode::Programm(nodes) => {
                for node in nodes {
                    self.analyze(node)?;
                }
                Ok(())
            }
            AstNode::FunctionDecl(data) => {
                let mut scope = self.scope_stack.last_mut().unwrap();
                if scope.contains_key(&data.name) {
                    return Err(SemanticError::Redefinition(format!(
                        "Function with {} already exists",
                        data.name
                    )));
                }
                let param_types = data.parameter.iter().map(|p| p.param_typ.clone()).collect();
                let func_meta_data = MetaData {
                    typ: data.return_type.clone(),
                    kind: ElementKind::Function,
                    parameters: Some(param_types),
                };
                scope.insert(data.name.clone(), func_meta_data);
                self.current_return_type = Some(data.return_type.clone());

                self.enter_scope();
                for param in &data.parameter {
                    let param_scope = self.scope_stack.last_mut().unwrap();
                    if param_scope.contains_key(&param.name) {
                        return Err(SemanticError::Redefinition(format!(
                            "Parameter '{}' already exists",
                            param.name
                        )));
                    }
                    param_scope.insert(
                        param.name.clone(),
                        MetaData {
                            typ: param.param_typ.clone(),
                            kind: ElementKind::Parameter,
                            parameters: None,
                        },
                    );
                }

                self.analyze(*data.body)?;
                self.exit_scope();
                Ok(())
            }
            AstNode::VarDeclStatement(data) => {
                if self.scope_stack.last().unwrap().contains_key(&data.name) {
                    return Err(SemanticError::Redefinition(format!(
                        "Variable '{}' already defined it this scope",
                        data.name
                    )));
                }

                if let Some(init) = &data.initializer {
                    let init_type = self.analyze_expr(init)?;
                    if init_type != data.var_typ {
                        return Err(SemanticError::NotMatchingReturnType(format!(
                            "Variable '{}' of type '{:?}' cannot be initialized with type '{:?}'",
                            data.name, data.var_typ, init_type
                        )));
                    }
                }

                let scope = self.scope_stack.last_mut().unwrap();
                scope.insert(
                    data.name.clone(),
                    MetaData {
                        typ: data.var_typ.clone(),
                        kind: ElementKind::Variable,
                        parameters: None,
                    },
                );
                Ok(())
            }
            AstNode::AssignmentStatement(data) => {
                let var_type = match self.lookup_variable(&data.name) {
                    Some(meta) => meta.typ.clone(),
                    None => {
                        return Err(SemanticError::UndefinedVariable(format!(
                            "Variable '{}' is not defined",
                            data.name
                        )));
                    }
                };

                let val_type = self.analyze_expr(&data.value)?;
                if val_type != var_type {
                    return Err(SemanticError::NotMatchingReturnType(format!(
                        "Cannot assign type '{:?}' to variable '{}' of type '{:?}'",
                        val_type, data.name, var_type
                    )));
                }
                Ok(())
            }
            AstNode::ReturnStatement(data) => {
                let return_type = match &self.current_return_type {
                    Some(rt) => rt.clone(),
                    None => {
                        return Err(SemanticError::InvalidReturnType(
                            "Return statement outside of function".to_string(),
                        ));
                    }
                };

                let expr_type = self.analyze_expr(&data)?;
                if expr_type == return_type {
                    return Ok(());
                }

                Err(SemanticError::NotMatchingReturnType(format!(
                    "Function returns type '{:?}' but expression is of type '{:?}'",
                    return_type, expr_type
                )))
            }
            AstNode::IntLiteralExpr(_)
            | AstNode::BoolLiteralExpr(_)
            | AstNode::FloatLiteralExpr(_)
            | AstNode::StringLiteralExpr(_) => Ok(()),
            AstNode::BlockStatement(nodes) => {
                self.enter_scope();
                for node in nodes {
                    self.analyze(node)?;
                }
                self.exit_scope();
                Ok(())
            }
            AstNode::IfElseStatement(data) => {
                let cond_type = self.analyze_expr(&data.condition_expr)?;
                if cond_type != Type::Bool {
                    return Err(SemanticError::NotMatchingReturnType(format!(
                        "If condition must be of type bool, found '{:?}'",
                        cond_type
                    )));
                }

                self.analyze(*data.if_branch)?;
                if let Some(else_branch) = data.else_branch {
                    self.analyze(*else_branch)?;
                }

                Ok(())
            }
            node => {
                return Err(SemanticError::NotImplemented(format!(
                    "Function not implemented for {:?}",
                    node
                )));
            }
        }
    }

    fn analyze_expr(&mut self, node: &AstNode) -> Result<Type, SemanticError> {
        match node {
            AstNode::IntLiteralExpr(_) => Ok(Type::Int),
            AstNode::BoolLiteralExpr(_) => Ok(Type::Bool),
            AstNode::FloatLiteralExpr(_) => Ok(Type::Float),
            AstNode::StringLiteralExpr(_) => Ok(Type::String),
            AstNode::VariableExpr(name) => match self.lookup_variable(name) {
                Some(meta) => Ok(meta.typ.clone()),
                None => Err(SemanticError::UndefinedVariable(format!(
                    "Variable '{}' is not defined",
                    name
                ))),
            },
            AstNode::BinaryExpr(data) => {
                let left_type = self.analyze_expr(&data.left)?;
                let right_type = self.analyze_expr(&data.right)?;

                if left_type != right_type {
                    return Err(SemanticError::NotMatchingReturnType(format!(
                        "Binary expression operands must be of type Int, found '{:?}' and '{:?}'",
                        left_type, right_type
                    )));
                }

                match &data.operator {
                    crate::parser::BinaryOperator::Add
                    | crate::parser::BinaryOperator::Sub
                    | crate::parser::BinaryOperator::Mul
                    | crate::parser::BinaryOperator::Div => {
                        if left_type != Type::Int && left_type != Type::Float {
                            return Err(SemanticError::NotMatchingReturnType(format!(
                                "Arithmetic operators require int or float operands, found {:?}",
                                left_type
                            )));
                        }
                        Ok(left_type)
                    }
                    crate::parser::BinaryOperator::LessThan
                    | crate::parser::BinaryOperator::GreaterThan
                    | crate::parser::BinaryOperator::LessOrEqual
                    | crate::parser::BinaryOperator::GreaterOrEqual
                    | crate::parser::BinaryOperator::DoubleEqual => Ok(Type::Bool),
                    other => Err(SemanticError::NotImplemented(format!(
                        "Binary operator not implemented: {:?}",
                        other
                    ))),
                }
            }
            AstNode::CallExpr(data) => {
                let (func_type, params) = {
                    let meta = match self.lookup_variable(&data.name) {
                        Some(m) => m,
                        None => {
                            return Err(SemanticError::UndefinedVariable(format!(
                                "Function '{}' is not defined",
                                data.name
                            )));
                        }
                    };

                    if !matches!(meta.kind, ElementKind::Function) {
                        return Err(SemanticError::NotMatchingReturnType(format!(
                            "'{}' is not a function",
                            data.name
                        )));
                    }

                    let params = meta
                        .parameters
                        .as_ref()
                        .ok_or_else(|| {
                            SemanticError::NotMatchingReturnType(format!(
                                "Function '{}' has no parameter metadata",
                                data.name
                            ))
                        })?
                        .clone();

                    (meta.typ.clone(), params)
                };

                if data.arguments.len() != params.len() {
                    return Err(SemanticError::NotMatchingReturnType(format!(
                        "Function '{}' expects {} arguments, but {} were provided",
                        data.name,
                        params.len(),
                        data.arguments.len()
                    )));
                }

                for (arg, expected_type) in data.arguments.iter().zip(params.iter()) {
                    let arg_type = self.analyze_expr(arg)?;
                    if &arg_type != expected_type {
                        return Err(SemanticError::NotMatchingReturnType(format!(
                            "Argument type mismatch: expected '{:?}', found '{:?}'",
                            expected_type, arg_type
                        )));
                    }
                }

                Ok(func_type)
            }
            other => Err(SemanticError::InvalidReturnType(format!(
                "Expression type not supported: {:?}",
                other
            ))),
        }
    }

    fn lookup_variable(&self, name: &str) -> Option<&MetaData> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(meta) = scope.get(name) {
                return Some(meta);
            }
        }
        None
    }

    fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }
}

#[cfg(test)]
mod tests {
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
                AstNode::VarDeclStatement(crate::parser::VarDeclData {
                    name: "x".to_string(),
                    var_typ: Type::Int,
                    initializer: Some(Box::new(AstNode::IntLiteralExpr(10))),
                }),
                AstNode::IfElseStatement(crate::parser::IfElseData {
                    condition_expr: Box::new(AstNode::BinaryExpr(crate::parser::BinaryExpData {
                        left: Box::new(AstNode::VariableExpr("x".to_string())),
                        right: Box::new(AstNode::IntLiteralExpr(5)),
                        operator: crate::parser::BinaryOperator::GreaterThan,
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
                crate::parser::IfElseData {
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
        use crate::parser::{BinaryExpData, BinaryOperator, CallData, Parameter};

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
                AstNode::VarDeclStatement(crate::parser::VarDeclData {
                    name: "isActive".to_string(),
                    var_typ: Type::Bool,
                    initializer: Some(Box::new(AstNode::BoolLiteralExpr(true))),
                }),
                AstNode::VarDeclStatement(crate::parser::VarDeclData {
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
                AstNode::VarDeclStatement(crate::parser::VarDeclData {
                    name: "x".to_string(),
                    var_typ: Type::Int,
                    initializer: Some(Box::new(AstNode::IntLiteralExpr(10))),
                }),
                AstNode::AssignmentStatement(crate::parser::AssignmentData {
                    name: "x".to_string(),
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
        use crate::parser::{BinaryExpData, BinaryOperator};

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
                AstNode::VarDeclStatement(crate::parser::VarDeclData {
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
}
