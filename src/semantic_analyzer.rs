use crate::parser::{AstNode, FunctionDeclData, Type};
use std::collections::HashMap;

#[derive(Debug)]
pub enum SemanticError {
    Redefinition(String),
    InvalidReturnType(String),
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
                scope.insert(data.name, func_meta_data);
                self.current_return_type = Some(data.return_type);
                self.analyze(*data.body)?;
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
}
