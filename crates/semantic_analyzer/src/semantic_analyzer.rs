use parser::{AstNode, Type};
use std::collections::HashMap;

#[derive(Debug)]
pub enum SemanticError {
    Redefinition {
        message: String,
        line: usize,
        column: usize,
    },
    UndefinedVariable {
        name: String,
        line: usize,
        column: usize,
    },
    UndefinedFunction {
        name: String,
        line: usize,
        column: usize,
    },
    UndefinedStruct {
        name: String,
        line: usize,
        column: usize,
    },
    TypeMismatch {
        expected: String,
        found: String,
        message: String,
        line: usize,
        column: usize,
    },
    InvalidReturnType {
        message: String,
        line: usize,
        column: usize,
    },
    NotAncillaryElement {
        name: String,
        expected: String,
        line: usize,
        column: usize,
    },
    NotImplemented {
        message: String,
        line: usize,
        column: usize,
    },
}

#[derive(Clone, Debug)]
enum ElementKind<'a> {
    Function { parameters: Vec<Type<'a>> },
    Variable,
    Parameter,
}

#[derive(Clone, Debug)]
struct MetaData<'a> {
    typ: Type<'a>,
    kind: ElementKind<'a>,
}

pub struct Analyzer<'a> {
    scope_stack: Vec<HashMap<String, MetaData<'a>>>,
    current_return_type: Option<Type<'a>>,
    struct_definitions: HashMap<String, HashMap<String, Type<'a>>>,
}

impl<'a> Analyzer<'a> {
    pub fn new() -> Self {
        Self {
            scope_stack: vec![HashMap::new()],
            current_return_type: None,
            struct_definitions: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, ast: &AstNode<'a>) -> Result<(), SemanticError> {
        match ast {
            AstNode::Programm(nodes) => {
                for node in nodes {
                    self.analyze(node)?;
                }
                Ok(())
            }
            AstNode::FunctionDecl(data) => {
                let param_types: Vec<Type<'a>> =
                    data.parameter.iter().map(|p| p.param_typ.clone()).collect();
                let func_meta = MetaData {
                    typ: data.return_type.clone(),
                    kind: ElementKind::Function {
                        parameters: param_types,
                    },
                };

                if self
                    .current_scope_mut()
                    .insert(data.name.to_string(), func_meta)
                    .is_some()
                {
                    return Err(SemanticError::Redefinition {
                        message: format!("Function '{}' already exists", data.name),
                        line: data.line,
                        column: data.column,
                    });
                }

                let prvious_return_type =
                    self.current_return_type.replace(data.return_type.clone());
                self.enter_scope();

                for param in &data.parameter {
                    let param_meta = MetaData {
                        typ: param.param_typ.clone(),
                        kind: ElementKind::Parameter,
                    };
                    if self
                        .current_scope_mut()
                        .insert(param.name.to_string(), param_meta)
                        .is_some()
                    {
                        return Err(SemanticError::Redefinition {
                            message: format!("Parameter '{}' already exists", param.name),
                            line: data.line,
                            column: data.column,
                        });
                    }
                }

                self.analyze(&data.body)?;
                self.exit_scope();
                self.current_return_type = prvious_return_type;

                Ok(())
            }
            AstNode::VarDeclStatement(data) => {
                if let Some(init) = &data.initializer {
                    let init_type = self.analyze_expr(init)?;
                    if init_type != data.var_typ {
                        return Err(SemanticError::TypeMismatch {
                            expected: format!("{:?}", data.var_typ),
                            found: format!("{:?}", init_type),
                            message: format!("Variable '{}' type mismatch", data.name),
                            line: data.line,
                            column: data.column,
                        });
                    }
                }

                let var_meta = MetaData {
                    typ: data.var_typ.clone(),
                    kind: ElementKind::Variable,
                };

                if self
                    .current_scope_mut()
                    .insert(data.name.to_string(), var_meta)
                    .is_some()
                {
                    return Err(SemanticError::Redefinition {
                        message: format!("Variable '{}' already defined in this scope", data.name),
                        line: data.line,
                        column: data.column,
                    });
                }
                Ok(())
            }
            AstNode::AssignmentStatement(data) => {
                let target_type = self.analyze_expr(&data.target)?;
                let val_type = self.analyze_expr(&data.value)?;

                if val_type != target_type {
                    return Err(SemanticError::TypeMismatch {
                        expected: format!("{:?}", target_type),
                        found: format!("{:?}", val_type),
                        message: "Assignment type mismatch".to_string(),
                        line: data.line,
                        column: data.column,
                    });
                }
                Ok(())
            }
            AstNode::ReturnStatement(data) => {
                let return_type = self.current_return_type.clone().ok_or_else(|| {
                    SemanticError::InvalidReturnType {
                        message: "Return statement outside of function".to_string(),
                        line: data.line,
                        column: data.column,
                    }
                })?;

                if let Some(val) = &data.value {
                    let expr_type = self.analyze_expr(val)?;
                    if expr_type != return_type {
                        return Err(SemanticError::TypeMismatch {
                            expected: format!("{:?}", return_type),
                            found: format!("{:?}", expr_type),
                            message: "Return type mismatch".to_string(),
                            line: data.line,
                            column: data.column,
                        });
                    }
                }
                Ok(())
            }
            AstNode::BlockStatement(data) => {
                self.enter_scope();
                for node in &data.statements {
                    self.analyze(&node)?;
                }
                self.exit_scope();
                Ok(())
            }
            AstNode::IfElseStatement(data) => {
                let cond_type = self.analyze_expr(&data.condition_expr)?;
                if cond_type != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Bool".to_string(),
                        found: format!("{:?}", cond_type),
                        message: "If condition must be of type bool".to_string(),
                        line: data.line,
                        column: data.column,
                    });
                }

                self.analyze(&data.if_branch)?;
                if let Some(else_branch) = &data.else_branch {
                    self.analyze(&else_branch)?;
                }
                Ok(())
            }
            AstNode::WhileStatement(data) => {
                let cond_type = self.analyze_expr(&data.condition_expr)?;
                if cond_type != Type::Bool {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Bool".to_string(),
                        found: format!("{:?}", cond_type),
                        message: "While condition must be of type bool".to_string(),
                        line: data.line,
                        column: data.column,
                    });
                }

                self.analyze(&data.body)?;
                Ok(())
            }
            AstNode::StructDecl(data) => {
                if self.struct_definitions.contains_key(data.name) {
                    return Err(SemanticError::Redefinition {
                        message: format!("Struct '{}' is already defined", data.name),
                        line: data.line,
                        column: data.column,
                    });
                }

                let mut fields_map = HashMap::new();
                for field in &data.fields {
                    if fields_map
                        .insert(field.name.to_string(), field.param_typ.clone())
                        .is_some()
                    {
                        return Err(SemanticError::Redefinition {
                            message: format!("Field '{}' is already defined in struct", field.name),
                            line: data.line,
                            column: data.column,
                        });
                    }
                }

                self.struct_definitions
                    .insert(data.name.to_string(), fields_map);
                Ok(())
            }
            AstNode::PrintStatement(expr) => {
                self.analyze_expr(&expr)?;
                Ok(())
            }
            AstNode::IntLiteralExpr(_)
            | AstNode::BoolLiteralExpr(_)
            | AstNode::FloatLiteralExpr(_)
            | AstNode::StringLiteralExpr(_) => Ok(()),
            node => Err(SemanticError::NotImplemented {
                message: format!("Statement not implemented for {:?}", node),
                line: 0,
                column: 0,
            }),
        }
    }

    fn analyze_expr(&mut self, node: &AstNode<'a>) -> Result<Type<'a>, SemanticError> {
        match node {
            AstNode::IntLiteralExpr(_) => Ok(Type::Int),
            AstNode::BoolLiteralExpr(_) => Ok(Type::Bool),
            AstNode::FloatLiteralExpr(_) => Ok(Type::Float),
            AstNode::StringLiteralExpr(_) => Ok(Type::String),
            AstNode::VariableExpr(name) => {
                let meta =
                    self.lookup_variable(name)
                        .ok_or_else(|| SemanticError::UndefinedVariable {
                            name: name.to_string(),
                            line: 0,
                            column: 0,
                        })?;
                Ok(meta.typ.clone())
            }
            AstNode::BinaryExpr(data) => {
                let left_type = self.analyze_expr(&data.left)?;
                let right_type = self.analyze_expr(&data.right)?;

                if left_type != right_type {
                    return Err(SemanticError::TypeMismatch {
                        expected: format!("{:?}", left_type),
                        found: format!("{:?}", right_type),
                        message: "Binary expression operands must have matching types".to_string(),
                        line: data.line,
                        column: data.column,
                    });
                }

                match &data.operator {
                    parser::BinaryOperator::Add
                    | parser::BinaryOperator::Sub
                    | parser::BinaryOperator::Mul
                    | parser::BinaryOperator::Div => {
                        if left_type != Type::Int && left_type != Type::Float {
                            return Err(SemanticError::TypeMismatch {
                                expected: "Int or Float".to_string(),
                                found: format!("{:?}", left_type),
                                message: "Arithmetic operators require int or float operands"
                                    .to_string(),
                                line: data.line,
                                column: data.column,
                            });
                        }
                        Ok(left_type)
                    }
                    parser::BinaryOperator::LessThan
                    | parser::BinaryOperator::GreaterThan
                    | parser::BinaryOperator::LessOrEqual
                    | parser::BinaryOperator::GreaterOrEqual
                    | parser::BinaryOperator::DoubleEqual => Ok(Type::Bool),
                    other => Err(SemanticError::NotImplemented {
                        message: format!("Binary operator not implemented: {:?}", other),
                        line: data.line,
                        column: data.column,
                    }),
                }
            }
            AstNode::CallExpr(data) => {
                let (func_type, params) = {
                    let meta = self.lookup_variable(data.name).ok_or_else(|| {
                        SemanticError::UndefinedFunction {
                            name: data.name.to_string(),
                            line: data.line,
                            column: data.column,
                        }
                    })?;
                    let ElementKind::Function { parameters } = &meta.kind else {
                        return Err(SemanticError::NotAncillaryElement {
                            name: data.name.to_string(),
                            expected: "function".to_string(),
                            line: data.line,
                            column: data.column,
                        });
                    };
                    (meta.typ.clone(), parameters.clone())
                };

                if data.arguments.len() != params.len() {
                    return Err(SemanticError::InvalidReturnType {
                        message: format!(
                            "Function '{}' expects {} arguments, but {} were provided",
                            data.name,
                            params.len(),
                            data.arguments.len()
                        ),
                        line: data.line,
                        column: data.column,
                    });
                }

                for (arg, expected_type) in data.arguments.iter().zip(params.iter()) {
                    let arg_type = self.analyze_expr(arg)?;
                    if arg_type != *expected_type {
                        return Err(SemanticError::TypeMismatch {
                            expected: format!("{:?}", expected_type),
                            found: format!("{:?}", arg_type),
                            message: "Argument type mismtach".to_string(),
                            line: data.line,
                            column: data.column,
                        });
                    }
                }

                Ok(func_type)
            }
            AstNode::MemberAccessExpr(data) => {
                let object_type = self.analyze_expr(&data.object)?;

                let Type::Struct(struct_name) = object_type else {
                    return Err(SemanticError::InvalidReturnType {
                        message: String::from(
                            "Member access '.' is not allowed on non-struct types",
                        ),
                        line: data.line,
                        column: data.column,
                    });
                };

                let fields = self.struct_definitions.get(struct_name).ok_or_else(|| {
                    SemanticError::UndefinedStruct {
                        name: struct_name.to_string(),
                        line: data.line,
                        column: data.column,
                    }
                })?;

                let field_type =
                    fields
                        .get(&data.member)
                        .ok_or_else(|| SemanticError::UndefinedVariable {
                            name: format!(
                                "Struct '{}' has no field named '{}'",
                                struct_name, data.member
                            ),
                            line: data.line,
                            column: data.column,
                        })?;

                Ok(field_type.clone())
            }
            AstNode::ArrayIndexExpr(data) => {
                let array_type = self.analyze_expr(&data.array)?;
                let index_type = self.analyze_expr(&data.index)?;

                if index_type != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Int".to_string(),
                        found: format!("{:?}", index_type),
                        message: "Array index must be of type Int".to_string(),
                        line: data.line,
                        column: data.column,
                    });
                }

                let Type::Array(inner_type) = array_type else {
                    return Err(SemanticError::InvalidReturnType {
                        message: "Indexing not allowed on non-array type".to_string(),
                        line: data.line,
                        column: data.column,
                    });
                };

                Ok(*inner_type)
            }
            other => Err(SemanticError::NotImplemented {
                message: format!("Expression type not supported: {:?}", other),
                line: 0,
                column: 0,
            }),
        }
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, MetaData<'a>> {
        self.scope_stack
            .last_mut()
            .expect("Scope stack should never be empty")
    }

    fn lookup_variable(&self, name: &str) -> Option<&MetaData<'a>> {
        self.scope_stack
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }

    fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new())
    }

    fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
