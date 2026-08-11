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
                let func_meta_data = MetaData {
                    typ: data.return_type.clone(),
                    kind: ElementKind::Function,
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
            AstNode::IntLiteralExpr(_) => Ok(()),
            AstNode::BlockStatement(nodes) => {
                self.enter_scope();
                for node in nodes {
                    self.analyze(node)?;
                }
                self.exit_scope();
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

                if left_type != Type::Int || right_type != Type::Int {
                    return Err(SemanticError::NotMatchingReturnType(format!(
                        "Binary expression operands must be of type Int, found '{:?}' and '{:?}'",
                        left_type, right_type
                    )));
                }

                Ok(Type::Int)
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
}
