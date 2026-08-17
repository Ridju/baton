use crate::parser::CallData;
use crate::parser::{AstNode, BinaryOperator, Type};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq)]
pub enum GeneratorError {
    GeneralError(String),
}

#[derive(Debug, PartialEq)]
struct VarInfo {
    offset: i32,
    typ: Type,
}

#[derive(Debug)]
pub struct Generator {
    buffer: String,
    local_vars: HashMap<String, VarInfo>,
    stack_offset: i32,
    label_count: usize,
    struct_layouts: HashMap<String, HashMap<String, i32>>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            buffer: String::new(),
            local_vars: HashMap::new(),
            stack_offset: -8,
            label_count: 0,
            struct_layouts: HashMap::new(),
        }
    }

    fn expr_type(&self, node: &AstNode) -> Type {
        match node {
            AstNode::IntLiteralExpr(_) => Type::Int,
            AstNode::BoolLiteralExpr(_) => Type::Bool,
            AstNode::FloatLiteralExpr(_) => Type::Float,
            AstNode::StringLiteralExpr(_) => Type::String,
            AstNode::VariableExpr(name) => self.local_vars.get(name).unwrap().typ.clone(),
            AstNode::MemberAccessExpr(data) => {
                let obj_type = self.expr_type(&data.object);
                if let Type::Struct(struct_name) = obj_type {
                    if let Some(fields) = self.struct_layouts.get(&struct_name) {
                        return Type::Int;
                    }
                }
                Type::Int
            }
            AstNode::BinaryExpr(data) => match data.operator {
                BinaryOperator::LessThan
                | BinaryOperator::GreaterThan
                | BinaryOperator::LessOrEqual
                | BinaryOperator::GreaterOrEqual
                | BinaryOperator::DoubleEqual => Type::Bool,
                _ => self.expr_type(&data.left),
            },
            AstNode::ArrayIndexExpr(data) => {
                let obj_type = self.expr_type(&data.array);
                match obj_type {
                    Type::Array(inner) => *inner,
                    _ => panic!("Index expression on non-array type"),
                }
            }
            _ => Type::Int,
        }
    }

    pub fn generate(&mut self, ast: AstNode) -> Result<(), GeneratorError> {
        match ast {
            AstNode::Programm(nodes) => {
                self.buffer.push_str(".global _main\n.text\n\n");
                for node in nodes {
                    self.generate(node)?
                }
                Ok(())
            }
            AstNode::FunctionDecl(func_data) => {
                self.local_vars.clear();
                self.stack_offset = -8;

                self.buffer.push_str(&format!("_{}:\n", func_data.name));
                self.buffer
                    .push_str("\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n");

                self.buffer.push_str("\tsub sp, sp, #128\n");

                for (i, param) in func_data.parameter.iter().enumerate() {
                    let offset = self.stack_offset;
                    if param.param_typ == Type::Float {
                        self.buffer
                            .push_str(&format!("\tstr d{}, [x29, #{}]\n", i, offset))
                    } else {
                        self.buffer
                            .push_str(&format!("\tstr x{}, [x29, #{}]\n", i, offset));
                    }

                    self.local_vars.insert(
                        param.name.clone(),
                        VarInfo {
                            offset,
                            typ: param.param_typ.clone(),
                        },
                    );
                    self.stack_offset -= 8;
                }

                self.generate(*func_data.body)?;
                Ok(())
            }
            AstNode::ReturnStatement(node) => {
                self.generate(*node)?;
                self.buffer.push_str("\tmov sp, x29\n");
                self.buffer.push_str("\tldp x29, x30, [sp], #16\n\tret\n");
                Ok(())
            }
            AstNode::IntLiteralExpr(num) => {
                self.buffer.push_str(&format!("\tmov x0, #{}\n", num));
                Ok(())
            }
            AstNode::BoolLiteralExpr(val) => {
                let int_val = if val { 1 } else { 0 };
                self.buffer.push_str(&format!("\tmov x0, #{}\n", int_val));
                Ok(())
            }
            AstNode::FloatLiteralExpr(num_str) => {
                let label_idx = self.label_count;
                self.label_count += 1;
                let float_label = format!("L_float_{}", label_idx);

                self.buffer.push_str("\t.section __TEXT,__const\n");
                self.buffer.push_str("\t.align 3\n");
                self.buffer.push_str(&format!("{}:\n", float_label));
                self.buffer.push_str(&format!("\t.double {}\n", num_str));
                self.buffer.push_str("\t.text\n");

                self.buffer
                    .push_str(&format!("\tadrp x16, {}@PAGE\n", float_label));
                self.buffer
                    .push_str(&format!("\tldr d0, [x16, {}@PAGEOFF]\n", float_label));
                Ok(())
            }
            AstNode::StringLiteralExpr(val) => {
                let label_idx = self.label_count;
                self.label_count += 1;
                let str_label = format!("L_str_{}", label_idx);

                self.buffer.push_str("\t.section __TEXT,__cstring\n");
                self.buffer.push_str(&format!("{}:\n", str_label));
                self.buffer.push_str(&format!("\t.asciz \"{}\"\n", val));
                self.buffer.push_str("\t.text\n");

                self.buffer.push_str(&format!("\tldr x0, ={}\n", str_label));
                Ok(())
            }
            AstNode::BlockStatement(nodes) => {
                for node in nodes {
                    self.generate(node)?;
                }
                Ok(())
            }
            AstNode::BinaryExpr(bin_data) => {
                let left_type = self.expr_type(&bin_data.left);

                if left_type == Type::Float {
                    self.generate(*bin_data.left)?;
                    self.buffer.push_str("\tstr d0, [sp, #-16]!\n");

                    self.generate(*bin_data.right)?;
                    self.buffer.push_str("\tldr d1, [sp], #16\n");

                    match bin_data.operator {
                        BinaryOperator::Add => self.buffer.push_str("\tfadd d0, d1, d0\n"),
                        BinaryOperator::Sub => self.buffer.push_str("\tfsub d0, d1, d0\n"),
                        BinaryOperator::Mul => self.buffer.push_str("\tfmul d0, d1, d0\n"),
                        BinaryOperator::Div => self.buffer.push_str("\tfdiv d0, d1, d0\n"),
                        BinaryOperator::DoubleEqual => {
                            self.buffer.push_str("\tfcmp d1, d0\n");
                            self.buffer.push_str("\tcset x0, eq\n");
                        }
                        BinaryOperator::LessThan => {
                            self.buffer.push_str("\tfcmp d1, d0\n");
                            self.buffer.push_str("\tcset x0, lt\n");
                        }
                        BinaryOperator::GreaterThan => {
                            self.buffer.push_str("\tfcmp d1, d0\n");
                            self.buffer.push_str("\tcset x0, gt\n");
                        }
                        BinaryOperator::LessOrEqual => {
                            self.buffer.push_str("\tfcmp d1, d0\n");
                            self.buffer.push_str("\tcset x0, le\n");
                        }
                        BinaryOperator::GreaterOrEqual => {
                            self.buffer.push_str("\tfcmp d1, d0\n");
                            self.buffer.push_str("\tcset x0, ge\n");
                        }
                    }
                } else {
                    self.generate(*bin_data.left)?;
                    self.buffer.push_str("\tstr x0, [sp, #-16]!\n");

                    self.generate(*bin_data.right)?;
                    self.buffer.push_str("\tldr x1, [sp], #16\n");

                    match bin_data.operator {
                        BinaryOperator::Add => {
                            self.buffer.push_str("\tadd x0, x1, x0\n");
                        }
                        BinaryOperator::Sub => {
                            self.buffer.push_str("\tsub x0, x1, x0\n");
                        }
                        BinaryOperator::Mul => {
                            self.buffer.push_str("\tmul x0, x1, x0\n");
                        }
                        BinaryOperator::Div => {
                            self.buffer.push_str("\tsdiv x0, x1, x0\n");
                        }
                        BinaryOperator::DoubleEqual => {
                            self.buffer.push_str("\tcmp x1, x0\n");
                            self.buffer.push_str("\tcset x0, eq\n");
                        }
                        BinaryOperator::LessThan => {
                            self.buffer.push_str("\tcmp x1, x0\n");
                            self.buffer.push_str("\tcset x0, lt\n");
                        }
                        BinaryOperator::GreaterThan => {
                            self.buffer.push_str("\tcmp x1, x0\n");
                            self.buffer.push_str("\tcset x0, gt\n");
                        }
                        BinaryOperator::LessOrEqual => {
                            self.buffer.push_str("\tcmp x1, x0\n");
                            self.buffer.push_str("\tcset x0, le\n");
                        }
                        BinaryOperator::GreaterOrEqual => {
                            self.buffer.push_str("\tcmp x1, x0\n");
                            self.buffer.push_str("\tcset x0, ge\n");
                        }
                    }
                }

                Ok(())
            }
            AstNode::VarDeclStatement(var_data) => {
                let is_float = var_data.var_typ == Type::Float;
                let is_struct = matches!(var_data.var_typ, Type::Struct(_));

                if let Some(init) = var_data.initializer {
                    self.generate(*init)?;
                } else {
                    if is_float {
                        self.buffer.push_str("\tfmov d0, #0.0\n");
                    } else if !is_struct {
                        self.buffer.push_str("\tmov x0, #0\n");
                    }
                }

                let offset = self.stack_offset;

                let size = match &var_data.var_typ {
                    Type::Struct(struct_name) => {
                        let fields = self
                            .struct_layouts
                            .get(struct_name)
                            .expect("Unknown struct");
                        (fields.len() * 8) as i32
                    }
                    Type::Array(inner) => 80,
                    _ => 8,
                };

                self.local_vars.insert(
                    var_data.name.clone(),
                    VarInfo {
                        offset,
                        typ: var_data.var_typ.clone(),
                    },
                );

                self.stack_offset -= size;

                if !is_struct && !matches!(var_data.var_typ, Type::Array(_)) {
                    if is_float {
                        self.buffer
                            .push_str(&format!("\tstr d0, [x29, #{}]\n", offset));
                    } else {
                        self.buffer
                            .push_str(&format!("\tstr x0, [x29, #{}]\n", offset));
                    }
                }
                Ok(())
            }
            AstNode::AssignmentStatement(assign_data) => {
                match &*assign_data.target {
                    AstNode::VariableExpr(name) => {
                        let var_info = self.local_vars.get(name).unwrap_or_else(|| {
                            panic!("Undefined variable '{}' in code generator", name);
                        });
                        let is_float = var_info.typ == Type::Float;
                        let offset = var_info.offset;

                        self.generate(*assign_data.value)?;

                        if is_float {
                            self.buffer
                                .push_str(&format!("\tstr d0, [x29, #{}]\n", offset));
                        } else {
                            self.buffer
                                .push_str(&format!("\tstr x0, [x29, #{}]\n", offset));
                        }
                    }
                    AstNode::MemberAccessExpr(member_data) => {
                        let obj_type = self.expr_type(&member_data.object);
                        let struct_name = match obj_type {
                            Type::Struct(name) => name,
                            _ => panic!("Member access on non-struct type"),
                        };

                        let offset = *self
                            .struct_layouts
                            .get(&struct_name)
                            .expect("Unknown struct")
                            .get(&member_data.member)
                            .expect("Unknown struct field");

                        self.generate(*member_data.object.clone())?;

                        self.buffer.push_str("\tstr x0, [sp, #-16]!\n");

                        self.generate(*assign_data.value)?;

                        self.buffer.push_str("\tldr x1, [sp], #16\n");
                        if offset != 0 {
                            self.buffer
                                .push_str(&format!("\tstr x0, [x1, #{}]\n", offset));
                        } else {
                            self.buffer.push_str("\tstr x0, [x1]\n");
                        }
                    }
                    AstNode::ArrayIndexExpr(array_data) => {
                        let array_type = self.expr_type(&array_data.array);
                        let inner_type = match array_type {
                            Type::Array(inner) => *inner,
                            _ => panic!("Index assigment on non-array type"),
                        };
                        let is_float = inner_type == Type::Float;

                        self.generate(*array_data.array.clone())?;
                        self.buffer.push_str("\tstr x0, [sp, #-16]!\n");

                        self.generate(*array_data.index.clone())?;
                        self.buffer.push_str("\tldr x1, [sp], #16\n");
                        self.buffer.push_str("\tlsl x0, x0, #3\n");
                        self.buffer.push_str("\tadd x0, x1, x0\n");

                        self.buffer.push_str("\tstr x0, [sp, #-16]!\n");

                        self.generate(*assign_data.value)?;

                        self.buffer.push_str("\tldr x1, [sp], #16\n");
                        if is_float {
                            self.buffer.push_str("\tstr d0, [x1]\n");
                        } else {
                            self.buffer.push_str("\tstr x0, [x1]\n");
                        }
                    }
                    _ => panic!("Invalid assignment target"),
                }
                Ok(())
            }
            AstNode::VariableExpr(name) => {
                let var_info = self.local_vars.get(&name).unwrap_or_else(|| {
                    panic!("Undefined variable '{}' in code generator", name);
                });
                let offset = var_info.offset;

                match &var_info.typ {
                    Type::Struct(_) | Type::Array(_) => {
                        self.buffer
                            .push_str(&format!("\tadd x0, x29, #{}\n", offset));
                    }
                    Type::Float => {
                        self.buffer
                            .push_str(&format!("\tldr d0, [x29, #{}]\n", offset));
                    }
                    _ => {
                        self.buffer
                            .push_str(&format!("\tldr x0, [x29, #{}]\n", offset));
                    }
                }

                Ok(())
            }
            AstNode::IfElseStatement(data) => {
                let label_idx = self.label_count;
                self.label_count += 1;

                let else_label = format!(".L_else_{}", label_idx);
                let end_label = format!(".L_end_{}", label_idx);

                self.generate(*data.condition_expr)?;

                let target_label = if data.else_branch.is_some() {
                    &else_label
                } else {
                    &end_label
                };
                self.buffer
                    .push_str(&format!("\tcbz x0, {}\n", target_label));

                self.generate(*data.if_branch)?;

                if let Some(ref else_node) = data.else_branch {
                    self.buffer.push_str(&format!("\tb {}\n", end_label));
                    self.buffer.push_str(&format!("{}:\n", else_label));
                    self.generate(*else_node.clone())?;
                }

                self.buffer.push_str(&format!("{}:\n", end_label));
                Ok(())
            }
            AstNode::CallExpr(data) => {
                for arg in &data.arguments {
                    let arg_type = self.expr_type(arg);
                    self.generate(arg.clone())?;
                    if arg_type == Type::Float {
                        self.buffer.push_str("\tstr d0, [sp, #-16]!\n");
                    } else {
                        self.buffer.push_str("\tstr x0, [sp, #-16]!\n");
                    }
                }

                for (i, arg) in data.arguments.iter().enumerate().rev() {
                    let arg_type = self.expr_type(arg);
                    if arg_type == Type::Float {
                        let reg = format!("d{}", i);
                        self.buffer.push_str(&format!("\tldr {}, [sp], #16\n", reg));
                    } else {
                        let reg = format!("x{}", i);
                        self.buffer.push_str(&format!("\tldr {}, [sp], #16\n", reg));
                    }
                }

                self.buffer.push_str(&format!("\tbl _{}\n", data.name));
                Ok(())
            }
            AstNode::StructDecl(struct_data) => {
                let mut field_offsets = HashMap::new();
                let mut current_offset = 0;
                for field in struct_data.fields {
                    field_offsets.insert(field.name, current_offset);
                    current_offset += 8;
                }
                self.struct_layouts
                    .insert(struct_data.name.clone(), field_offsets);
                Ok(())
            }
            AstNode::MemberAccessExpr(member_data) => {
                self.generate(*member_data.object.clone())?;
                let obj_type = self.expr_type(&member_data.object);
                let struct_name = match obj_type {
                    Type::Struct(name) => name,
                    _ => panic!("Member access on non-struct type"),
                };

                let fields = self
                    .struct_layouts
                    .get(&struct_name)
                    .expect("Unknown struct");
                let offset = fields
                    .get(&member_data.member)
                    .expect("Unknown struct field");

                if *offset != 0 {
                    self.buffer
                        .push_str(&format!("\tldr x0, [x0, #{}]\n", offset));
                } else {
                    self.buffer.push_str("\tldr x0, [x0]\n");
                }

                Ok(())
            }
            AstNode::ArrayIndexExpr(data) => {
                let array_type = self.expr_type(&data.array);
                let inner_type = match array_type {
                    Type::Array(inner) => *inner,
                    _ => panic!("Index expression on non-array type"),
                };

                self.generate(*data.array.clone())?;
                self.buffer.push_str("\tstr x0, [sp, #-16]!\n");

                self.generate(*data.index.clone())?;

                self.buffer.push_str("\tldr x1, [sp], #16\n");

                self.buffer.push_str("\tlsl x0, x0, #3\n");

                self.buffer.push_str("\tadd x0, x1, x0\n");

                if inner_type == Type::Float {
                    self.buffer.push_str("\tldr d0, [x0]\n");
                } else {
                    self.buffer.push_str("\tldr x0, [x0]\n");
                }

                Ok(())
            }
        }
    }

    fn write_to_file(&self, path: &str) -> io::Result<()> {
        fs::write(path, &self.buffer)
    }

    pub fn compile_executable(
        &self,
        input_file: &str,
        output_name: &str,
    ) -> Result<(), GeneratorError> {
        let output_dir = Path::new("output");
        fs::create_dir_all(output_dir).map_err(|e| {
            GeneratorError::GeneralError(format!("Could not create output directory: {}", e))
        })?;

        let input_path = Path::new(input_file);
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let s_filename = format!("{}.s", file_stem);

        let s_path = output_dir.join(&s_filename);
        let output_path = output_dir.join(output_name);

        self.write_to_file(s_path.to_str().unwrap())
            .map_err(|e| GeneratorError::GeneralError(format!("Could not write file: {}", e)))?;

        let status = Command::new("cc")
            .arg(&s_path)
            .arg("-o")
            .arg(&output_path)
            .status()
            .map_err(|e| {
                GeneratorError::GeneralError(format!("Compiler could not be started: {}", e))
            })?;

        let _ = fs::remove_file(&s_path);

        if status.success() {
            Ok(())
        } else {
            Err(GeneratorError::GeneralError(
                "Compiler error (compilation failed)".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ArrayIndexData, AstNode, BinaryExpData, FunctionDeclData, Type};

    #[test]
    fn test_generate_simple_main() {
        let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(42)),
            )])),
        })]);

        let mut generator = Generator::new();
        generator.generate(ast).unwrap();

        let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #128\n\tmov x0, #42\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n";

        assert_eq!(generator.buffer, expected_assembly);
    }

    #[test]
    fn test_generate_zero_return() {
        let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::IntLiteralExpr(0)),
            )])),
        })]);

        let mut generator = Generator::new();
        generator.generate(ast).unwrap();
        let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #128\n\tmov x0, #0\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n";

        assert_eq!(generator.buffer, expected_assembly);
    }

    #[test]
    fn test_generate_multiple_functions() {
        let ast = AstNode::Programm(vec![
            AstNode::FunctionDecl(FunctionDeclData {
                name: "main".to_string(),
                return_type: Type::Int,
                parameter: Vec::new(),
                body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                    Box::new(AstNode::IntLiteralExpr(0)),
                )])),
            }),
            AstNode::FunctionDecl(FunctionDeclData {
                name: "helper_func".to_string(),
                return_type: Type::Int,
                parameter: Vec::new(),
                body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                    Box::new(AstNode::IntLiteralExpr(100)),
                )])),
            }),
        ]);

        let mut generator = Generator::new();
        generator.generate(ast).unwrap();

        let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #128\n\tmov x0, #0\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n_helper_func:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #128\n\tmov x0, #100\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n";

        assert_eq!(generator.buffer, expected_assembly);
    }

    #[test]
    fn test_generate_block_with_multiple_statements() {
        let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![
                AstNode::IntLiteralExpr(5),
                AstNode::ReturnStatement(Box::new(AstNode::IntLiteralExpr(10))),
            ])),
        })]);

        let mut generator = Generator::new();
        generator.generate(ast).unwrap();

        let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tsub sp, sp, #128\n\tmov x0, #5\n\tmov x0, #10\n\tmov sp, x29\n\tldp x29, x30, [sp], #16\n\tret\n";

        assert_eq!(generator.buffer, expected_assembly);
    }

    #[test]
    fn test_codegen_binary_expression() {
        let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::BinaryExpr(BinaryExpData {
                    left: Box::new(AstNode::IntLiteralExpr(200)),
                    right: Box::new(AstNode::IntLiteralExpr(20)),
                    operator: BinaryOperator::Add,
                })),
            )])),
        })]);

        let mut codegen = Generator::new();
        let assembly = codegen.generate(ast).unwrap();

        assert!(
            codegen.buffer.contains(".global _main") || codegen.buffer.contains(".global main")
        );
        assert!(codegen.buffer.contains("add"));
        assert!(codegen.buffer.contains("str"));
        assert!(codegen.buffer.contains("ldr"));
        assert!(codegen.buffer.contains("ret"));
    }

    #[test]
    fn test_codegen_if_else_statement() {
        use crate::parser::{BinaryExpData, BinaryOperator, IfElseData};

        let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::IfElseStatement(
                IfElseData {
                    condition_expr: Box::new(AstNode::BinaryExpr(BinaryExpData {
                        left: Box::new(AstNode::IntLiteralExpr(10)),
                        right: Box::new(AstNode::IntLiteralExpr(5)),
                        operator: BinaryOperator::GreaterThan,
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
        use crate::parser::{BinaryExpData, BinaryOperator, Parameter};

        let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
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
        use crate::parser::CallData;

        let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: vec![],
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::CallExpr(CallData {
                    name: "add".to_string(),
                    arguments: vec![AstNode::IntLiteralExpr(1), AstNode::IntLiteralExpr(3)],
                })),
            )])),
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
            name: "main".to_string(),
            return_type: Type::Bool,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::BoolLiteralExpr(true)),
            )])),
        })]);

        let mut generator = Generator::new();
        generator.generate(ast).unwrap();

        let assembly = generator.buffer;
        assert!(assembly.contains("mov x0, #1"));
    }

    #[test]
    fn test_codegen_float_literal() {
        let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Float,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::FloatLiteralExpr(3.14)),
            )])),
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
            name: "main".to_string(),
            return_type: Type::String,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::StringLiteralExpr("Hello World".to_string())),
            )])),
        })]);

        let mut generator = Generator::new();
        generator.generate(ast).unwrap();

        let assembly = generator.buffer;
        assert!(assembly.contains("__TEXT,__cstring"));
        assert!(assembly.contains(".asciz \"Hello World\""));
        assert!(assembly.contains("ldr x0, =L_str_0"));
    }

    #[test]
    fn test_codegen_float_binary_expression() {
        let ast = AstNode::Programm(vec![AstNode::FunctionDecl(FunctionDeclData {
            name: "main".to_string(),
            return_type: Type::Float,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![AstNode::ReturnStatement(
                Box::new(AstNode::BinaryExpr(BinaryExpData {
                    left: Box::new(AstNode::FloatLiteralExpr(1.5)),
                    right: Box::new(AstNode::FloatLiteralExpr(2.5)),
                    operator: BinaryOperator::Add,
                })),
            )])),
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
        use crate::parser::{MemberAccessData, Parameter, StructDeclData};

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
                    AstNode::VarDeclStatement(crate::parser::VarDeclData {
                        name: "p".to_string(),
                        var_typ: Type::Struct("Point".to_string()),
                        initializer: None,
                    }),
                    AstNode::AssignmentStatement(crate::parser::AssignmentData {
                        target: Box::new(AstNode::MemberAccessExpr(MemberAccessData {
                            object: Box::new(AstNode::VariableExpr("p".to_string())),
                            member: "y".to_string(),
                        })),
                        value: Box::new(AstNode::IntLiteralExpr(10)),
                    }),
                    AstNode::ReturnStatement(Box::new(AstNode::MemberAccessExpr(
                        MemberAccessData {
                            object: Box::new(AstNode::VariableExpr("p".to_string())),
                            member: "y".to_string(),
                        },
                    ))),
                ])),
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
            name: "main".to_string(),
            return_type: Type::Int,
            parameter: Vec::new(),
            body: Box::new(AstNode::BlockStatement(vec![
                AstNode::VarDeclStatement(crate::parser::VarDeclData {
                    name: "a".to_string(),
                    var_typ: Type::Int,
                    initializer: Some(Box::new(AstNode::IntLiteralExpr(99))),
                }),
                AstNode::ReturnStatement(Box::new(AstNode::VariableExpr("a".to_string()))),
            ])),
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
        use crate::parser::{ArrayIndexData, VarDeclData};

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
                    index: Box::new(AstNode::IntLiteralExpr(2)),
                }))),
            ])),
        })]);

        let mut generator = Generator::new();
        generator.generate(ast).unwrap();

        let assembly = generator.buffer;
        assert!(assembly.contains("lsl x0, x0, #3"));
        assert!(assembly.contains("add x0, x1, x0"));
        assert!(assembly.contains("ldr x0, [x0]"));
    }
}
