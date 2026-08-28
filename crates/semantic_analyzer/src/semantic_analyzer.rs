use parser::{AstNode, FunctionDeclData, Type};
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
    struct_definitions: HashMap<String, HashMap<String, Type>>,
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            scope_stack: vec![HashMap::new()],
            current_return_type: None,
            struct_definitions: HashMap::new(),
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
                let target_type = self.analyze_expr(&data.target)?;

                let val_type = self.analyze_expr(&data.value)?;
                if val_type != target_type {
                    return Err(SemanticError::NotMatchingReturnType(format!(
                        "Cannot assign type '{:?}' to target of type '{:?}'",
                        val_type, target_type
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
            AstNode::WhileStatement(data) => {
                let cond_type = self.analyze_expr(&data.condition_expr)?;
                if cond_type != Type::Bool {
                    return Err(SemanticError::NotMatchingReturnType(format!(
                        "While ocndition must be of type bool, found '{:?}'",
                        cond_type
                    )));
                }

                self.analyze(*data.body)?;

                Ok(())
            }
            AstNode::StructDecl(data) => {
                if self.struct_definitions.contains_key(&data.name) {
                    return Err(SemanticError::Redefinition(format!(
                        "Struct '{}' is already defined",
                        data.name
                    )));
                }

                let mut fields_map = HashMap::new();
                for field in data.fields {
                    if fields_map.contains_key(&field.name) {
                        return Err(SemanticError::Redefinition(format!(
                            "Field '{}' is already defined in struct '{}'",
                            field.name, data.name
                        )));
                    }
                    fields_map.insert(field.name, field.param_typ);
                }

                self.struct_definitions
                    .insert(data.name.clone(), fields_map);
                Ok(())
            }
            AstNode::PrintStatement(expr) => {
                self.analyze_expr(&expr)?;
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
                    parser::BinaryOperator::Add
                    | parser::BinaryOperator::Sub
                    | parser::BinaryOperator::Mul
                    | parser::BinaryOperator::Div => {
                        if left_type != Type::Int && left_type != Type::Float {
                            return Err(SemanticError::NotMatchingReturnType(format!(
                                "Arithmetic operators require int or float operands, found {:?}",
                                left_type
                            )));
                        }
                        Ok(left_type)
                    }
                    parser::BinaryOperator::LessThan
                    | parser::BinaryOperator::GreaterThan
                    | parser::BinaryOperator::LessOrEqual
                    | parser::BinaryOperator::GreaterOrEqual
                    | parser::BinaryOperator::DoubleEqual => Ok(Type::Bool),
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
            AstNode::MemberAccessExpr(data) => {
                let object_type = self.analyze_expr(&data.object)?;

                let struct_name = match object_type {
                    Type::Struct(name) => name,
                    other => {
                        return Err(SemanticError::NotMatchingReturnType(format!(
                            "Member access '.' is not allowed on non-struct type '{:?}'",
                            other
                        )));
                    }
                };

                let fields = match self.struct_definitions.get(&struct_name) {
                    Some(f) => f,
                    None => {
                        return Err(SemanticError::UndefinedVariable(format!(
                            "Unknown struct type '{}'",
                            struct_name
                        )));
                    }
                };

                match fields.get(&data.member) {
                    Some(field_type) => Ok(field_type.clone()),
                    None => Err(SemanticError::UndefinedVariable(format!(
                        "Struct '{}' has no field name '{}'",
                        struct_name, data.member
                    ))),
                }
            }
            AstNode::ArrayIndexExpr(data) => {
                let array_type = self.analyze_expr(&data.array)?;
                let index_type = self.analyze_expr(&data.index)?;

                if index_type != Type::Int {
                    return Err(SemanticError::NotMatchingReturnType(format!(
                        "Array index must be of type Int, found '{:?}'",
                        index_type
                    )));
                }

                match array_type {
                    Type::Array(inner_type) => Ok(*inner_type),
                    other => Err(SemanticError::NotMatchingReturnType(format!(
                        "Indexing is not allowed on non-array type '{:?}'",
                        other
                    ))),
                }
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
#[path = "tests.rs"]
mod tests;
