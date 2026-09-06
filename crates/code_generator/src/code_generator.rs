use parser::{AstNode, BinaryOperator, Type};
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
struct VarInfo<'a> {
    offset: i32,
    typ: Type<'a>,
}

#[derive(Debug)]
pub struct Generator<'a> {
    buffer: String,
    local_vars: HashMap<String, VarInfo<'a>>,
    stack_offset: i32,
    label_count: usize,
    struct_layouts: HashMap<String, HashMap<String, (i32, Type<'a>)>>,
}

impl<'a> Default for Generator<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Generator<'a> {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            local_vars: HashMap::new(),
            stack_offset: -8,
            label_count: 0,
            struct_layouts: HashMap::new(),
        }
    }

    fn type_size(
        typ: &Type<'a>,
        struct_layouts: &HashMap<String, HashMap<String, (i32, Type<'a>)>>,
    ) -> i32 {
        match typ {
            Type::Int | Type::Bool | Type::Float | Type::String => 8,
            Type::Struct(name) => {
                if let Some(fields) = struct_layouts.get(*name) {
                    fields
                        .values()
                        .map(|(_, t)| Self::type_size(t, struct_layouts))
                        .sum()
                } else {
                    8
                }
            }
            Type::Array(inner) => Self::type_size(inner, struct_layouts) * 10,
        }
    }

    pub fn generate(&mut self, ast: AstNode<'a>) -> Result<(), GeneratorError> {
        match ast {
            AstNode::Programm(nodes) => self.gen_program(nodes),
            AstNode::FunctionDecl(func) => self.gen_function(func),
            AstNode::ReturnStatement(data) => self.gen_return(data),
            AstNode::IntLiteralExpr(num) => self.gen_int_literal(num),
            AstNode::BoolLiteralExpr(val) => self.gen_bool_literal(val),
            AstNode::FloatLiteralExpr(num) => self.gen_float_literal(num),
            AstNode::StringLiteralExpr(val) => self.gen_string_literal(val),
            AstNode::BlockStatement(nodes) => self.gen_block(nodes),
            AstNode::BinaryExpr(data) => self.gen_binary_expr(data),
            AstNode::VarDeclStatement(data) => self.gen_var_decl(data),
            AstNode::AssignmentStatement(data) => self.gen_assignment(data),
            AstNode::VariableExpr(name) => self.gen_variable(name),
            AstNode::IfElseStatement(data) => self.gen_if_else(data),
            AstNode::CallExpr(data) => self.gen_call(data),
            AstNode::StructDecl(data) => self.gen_struct_decl(data),
            AstNode::MemberAccessExpr(data) => self.gen_member_access(data),
            AstNode::ArrayIndexExpr(data) => self.gen_array_index(data),
            AstNode::WhileStatement(data) => self.gen_while(data),
            AstNode::PrintStatement(expr) => self.gen_print(*expr),
        }
    }

    fn gen_program(&mut self, nodes: Vec<AstNode<'a>>) -> Result<(), GeneratorError> {
        self.buffer.push_str(".global main\n.text\n\n");
        for node in nodes {
            self.generate(node)?;
        }
        Ok(())
    }

    fn gen_function(&mut self, func: parser::FunctionDeclData<'a>) -> Result<(), GeneratorError> {
        self.local_vars.clear();
        self.stack_offset = -8;

        self.buffer.push_str(&format!("{}:\n", func.name));
        self.buffer
            .push_str("\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n");
        self.buffer.push_str("\tsub sp, sp, #256\n");

        for (i, param) in func.parameter.iter().enumerate() {
            let offset = self.stack_offset;
            if param.param_typ == Type::Float {
                self.buffer
                    .push_str(&format!("\tstr d{}, [x29, #{}]\n", i, offset));
            } else {
                self.buffer
                    .push_str(&format!("\tstr x{}, [x29, #{}]\n", i, offset));
            }
            self.local_vars.insert(
                param.name.to_string(),
                VarInfo {
                    offset,
                    typ: param.param_typ.clone(),
                },
            );
            self.stack_offset -= 8;
        }

        self.generate(*func.body)?;
        Ok(())
    }

    fn gen_return(&mut self, data: parser::ReturnData<'a>) -> Result<(), GeneratorError> {
        if let Some(val) = data.value {
            self.generate(*val)?;
        } else {
            self.buffer.push_str("\tmov x0, #0\n");
        }
        self.buffer.push_str("\tmov sp, x29\n");
        self.buffer.push_str("\tldp x29, x30, [sp], #16\n\tret\n");
        Ok(())
    }

    fn gen_int_literal(&mut self, num: i16) -> Result<(), GeneratorError> {
        self.buffer.push_str(&format!("\tmov x0, #{}\n", num));
        Ok(())
    }

    fn gen_bool_literal(&mut self, val: bool) -> Result<(), GeneratorError> {
        let int_val = if val { 1 } else { 0 };
        self.buffer.push_str(&format!("\tmov x0, #{}\n", int_val));
        Ok(())
    }

    fn gen_float_literal(&mut self, num_str: f64) -> Result<(), GeneratorError> {
        let label_idx = self.label_count;
        self.label_count += 1;
        let float_label = format!("L_float_{}", label_idx);

        self.buffer.push_str("\t.section .rodata\n");
        self.buffer.push_str("\t.align 3\n");
        self.buffer.push_str(&format!("{}:\n", float_label));
        self.buffer.push_str(&format!("\t.double {}\n", num_str));
        self.buffer.push_str("\t.text\n");

        self.buffer
            .push_str(&format!("\tadrp x16, {}\n", float_label));
        self.buffer
            .push_str(&format!("\tldr d0, [x16, :lo12:{}]\n", float_label));
        Ok(())
    }

    fn gen_string_literal(&mut self, val: &str) -> Result<(), GeneratorError> {
        let label_idx = self.label_count;
        self.label_count += 1;
        let str_label = format!("L_str_{}", label_idx);

        self.buffer.push_str("\t.section .rodata\n");
        self.buffer.push_str(&format!("{}:\n", str_label));
        self.buffer.push_str(&format!("\t.asciz \"{}\"\n", val));
        self.buffer.push_str("\t.text\n");

        self.buffer.push_str(&format!("\tadrp x0, {}\n", str_label));
        self.buffer
            .push_str(&format!("\tadd x0, x0, :lo12:{}\n", str_label));

        Ok(())
    }

    fn gen_block(&mut self, data: parser::BlockStatementData<'a>) -> Result<(), GeneratorError> {
        for node in data.statements {
            self.generate(node)?;
        }

        Ok(())
    }

    fn gen_binary_expr(&mut self, data: parser::BinaryExpData<'a>) -> Result<(), GeneratorError> {
        let left_type = self.expr_type(&data.left);

        if left_type == Type::Float {
            self.generate(*data.left)?;
            self.buffer.push_str("\tstr d0, [sp, #-16]!\n");
            self.generate(*data.right)?;
            self.buffer.push_str("\tldr d1, [sp], #16\n");

            match data.operator {
                BinaryOperator::Add => self.buffer.push_str("\tfadd d0, d1, d0\n"),
                BinaryOperator::Sub => self.buffer.push_str("\tfsub d0, d1, d0\n"),
                BinaryOperator::Mul => self.buffer.push_str("\tfmul d0, d1, d0\n"),
                BinaryOperator::Div => self.buffer.push_str("\tfdiv d0, d1, d0\n"),
                BinaryOperator::DoubleEqual => {
                    self.buffer.push_str("\tfcmp d1, d0\n\tcset x0, eq\n");
                }
                BinaryOperator::LessThan => {
                    self.buffer.push_str("\tfcmp d1, d0\n\tcset x0, lt\n");
                }
                BinaryOperator::GreaterThan => {
                    self.buffer.push_str("\tfcmp d1, d0\n\tcset x0, gt\n");
                }
                BinaryOperator::LessOrEqual => {
                    self.buffer.push_str("\tfcmp d1, d0\n\tcset x0, le\n");
                }
                BinaryOperator::GreaterOrEqual => {
                    self.buffer.push_str("\tfcmp d1, d0\n\tcset x0, ge\n");
                }
            }
        } else {
            self.generate(*data.left)?;
            self.buffer.push_str("\tstr x0, [sp, #-16]!\n");
            self.generate(*data.right)?;
            self.buffer.push_str("\tldr x1, [sp], #16\n");

            match data.operator {
                BinaryOperator::Add => self.buffer.push_str("\tadd x0, x1, x0\n"),
                BinaryOperator::Sub => self.buffer.push_str("\tsub x0, x1, x0\n"),
                BinaryOperator::Mul => self.buffer.push_str("\tmul x0, x1, x0\n"),
                BinaryOperator::Div => self.buffer.push_str("\tsdiv x0, x1, x0\n"),
                BinaryOperator::DoubleEqual => {
                    self.buffer.push_str("\tcmp x1, x0\n\tcset x0, eq\n")
                }
                BinaryOperator::LessThan => self.buffer.push_str("\tcmp x1, x0\n\tcset x0, lt\n"),
                BinaryOperator::GreaterThan => {
                    self.buffer.push_str("\tcmp x1, x0\n\tcset x0, gt\n")
                }
                BinaryOperator::LessOrEqual => {
                    self.buffer.push_str("\tcmp x1, x0\n\tcset x0, le\n")
                }
                BinaryOperator::GreaterOrEqual => {
                    self.buffer.push_str("\tcmp x1, x0\n\tcset x0, ge\n")
                }
            }
        }
        Ok(())
    }

    fn gen_var_decl(&mut self, data: parser::VarDeclData<'a>) -> Result<(), GeneratorError> {
        let is_float = data.var_typ == Type::Float;

        if let Some(init) = data.initializer {
            self.generate(*init)?;
        } else {
            if is_float {
                self.buffer.push_str("\tfmov d0, #0.0\n");
            } else {
                self.buffer.push_str("\tmov x0, #0\n");
            }
        }

        let size = Self::type_size(&data.var_typ, &self.struct_layouts);
        self.stack_offset -= size;
        let offset = self.stack_offset;

        self.local_vars.insert(
            data.name.to_string(),
            VarInfo {
                offset,
                typ: data.var_typ.clone(),
            },
        );

        self.stack_offset -= size;

        if is_float {
            self.buffer
                .push_str(&format!("\tstr d0, [x29, #{}]\n", offset));
        } else if !matches!(data.var_typ, Type::Struct(_) | Type::Array(_)) {
            self.buffer
                .push_str(&format!("\tstr x0, [x29, #{}]\n", offset));
        }

        Ok(())
    }

    fn gen_variable(&mut self, name: &str) -> Result<(), GeneratorError> {
        let var_info = self.local_vars.get(name).unwrap();
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

    fn gen_assignment(&mut self, data: parser::AssignmentData<'a>) -> Result<(), GeneratorError> {
        match *data.target {
            AstNode::VariableExpr(name) => {
                let var_info = self.local_vars.get(name).unwrap();
                let is_float = var_info.typ == Type::Float;
                let offset = var_info.offset;

                self.generate(*data.value)?;

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
                    _ => panic!("Member access on non-struct"),
                };

                let (offset, field_type) = {
                    let fields = self.struct_layouts.get(struct_name).unwrap();
                    let (off, ft) = fields.get(&member_data.member).unwrap();
                    (*off, ft.clone())
                };

                self.generate(*member_data.object)?;
                self.buffer.push_str("\tstr x0, [sp, #-16]!\n");

                self.generate(*data.value)?;

                self.buffer.push_str("\tldr x1, [sp], #16\n");
                if field_type == Type::Float {
                    self.buffer
                        .push_str(&format!("\tstr d0, [x1, #{}]\n", offset));
                } else {
                    self.buffer
                        .push_str(&format!("\tstr x0, [x1, #{}]\n", offset));
                }
            }
            AstNode::ArrayIndexExpr(array_data) => {
                let array_type = self.expr_type(&array_data.array);
                let inner_type = match array_type {
                    Type::Array(inner) => *inner,
                    _ => panic!("Assignment index on non-array"),
                };

                self.generate(*array_data.array)?;
                self.buffer.push_str("\tstr x0, [sp, #-16]!\n");
                self.generate(*array_data.index)?;
                self.buffer.push_str("\tldr x1, [sp], #16\n");

                self.buffer.push_str("\tmov x2, #8\n");
                self.buffer.push_str("\tmul x0, x0, x2\n");
                self.buffer.push_str("\tadd x0, x1, x0\n");
                self.buffer.push_str("\tstr x0, [sp, #-16]!\n");

                self.generate(*data.value)?;

                self.buffer.push_str("\tldr x1, [sp], #16\n");
                if inner_type == Type::Float {
                    self.buffer.push_str("\tstr d0, [x1]\n");
                } else {
                    self.buffer.push_str("\tstr x0, [x1]\n");
                }
            }
            _ => panic!("Invalid assignment target"),
        }
        Ok(())
    }

    fn gen_if_else(&mut self, data: parser::IfElseData<'a>) -> Result<(), GeneratorError> {
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

        if let Some(else_node) = data.else_branch {
            self.buffer.push_str(&format!("\tb {}\n", end_label));
            self.buffer.push_str(&format!("{}:\n", else_label));
            self.generate(*else_node)?;
        }

        self.buffer.push_str(&format!("{}:\n", end_label));
        Ok(())
    }

    fn gen_call(&mut self, data: parser::CallData<'a>) -> Result<(), GeneratorError> {
        let arg_count = data.arguments.len();

        for arg in data.arguments {
            let arg_type = self.expr_type(&arg);
            self.generate(arg)?;
            if arg_type == Type::Float {
                self.buffer.push_str("\tstr d0, [sp, #-16]!\n");
            } else {
                self.buffer.push_str("\tstr x0, [sp, #-16]!\n");
            }
        }

        for i in (0..arg_count).rev() {
            self.buffer.push_str(&format!("\tldr x{}, [sp], #16\n", i));
        }

        self.buffer.push_str(&format!("\tbl {}\n", data.name));
        Ok(())
    }

    fn gen_struct_decl(&mut self, data: parser::StructDeclData<'a>) -> Result<(), GeneratorError> {
        let mut field_layouts = HashMap::new();
        let mut current_offset = 0;
        for field in data.fields {
            let size = Self::type_size(&field.param_typ, &self.struct_layouts);
            field_layouts.insert(field.name.to_string(), (current_offset, field.param_typ));
            current_offset += size;
        }
        self.struct_layouts
            .insert(data.name.to_string(), field_layouts);
        Ok(())
    }

    fn gen_member_access(
        &mut self,
        data: parser::MemberAccessData<'a>,
    ) -> Result<(), GeneratorError> {
        let obj_type = self.expr_type(&data.object);
        let struct_name = match obj_type {
            Type::Struct(name) => name,
            _ => panic!("Member access on non-struct"),
        };

        let (offset, field_type) = {
            let fields = self.struct_layouts.get(struct_name).unwrap();
            let (off, ft) = fields.get(&data.member).unwrap();
            (*off, ft.clone())
        };

        self.generate(*data.object)?;

        if field_type == Type::Float {
            self.buffer
                .push_str(&format!("\tldr d0, [x0, #{}]\n", offset));
        } else {
            self.buffer
                .push_str(&format!("\tldr x0, [x0, #{}]\n", offset));
        }
        Ok(())
    }

    fn gen_array_index(&mut self, data: parser::ArrayIndexData<'a>) -> Result<(), GeneratorError> {
        let array_type = self.expr_type(&data.array);
        let inner_type = match array_type {
            Type::Array(inner) => *inner,
            _ => panic!("Index expression on non-array"),
        };

        self.generate(*data.array)?;
        self.buffer.push_str("\tstr x0, [sp, #-16]!\n");
        self.generate(*data.index)?;
        self.buffer.push_str("\tldr x1, [sp], #16\n");

        self.buffer.push_str("\tmov x2, #8\n");
        self.buffer.push_str("\tmul x0, x0, x2\n");
        self.buffer.push_str("\tadd x0, x1, x0\n");

        if inner_type == Type::Float {
            self.buffer.push_str("\tldr d0, [x0]\n");
        } else {
            self.buffer.push_str("\tldr x0, [x0]\n");
        }
        Ok(())
    }

    fn gen_while(&mut self, data: parser::WhileLoopData<'a>) -> Result<(), GeneratorError> {
        let label_idx = self.label_count;
        self.label_count += 1;

        let start_label = format!(".L_while_start_{}", label_idx);
        let end_label = format!(".L_while_end_{}", label_idx);

        self.buffer.push_str(&format!("{}:\n", start_label));
        self.generate(*data.condition_expr)?;
        self.buffer.push_str(&format!("\tcbz x0, {}\n", end_label));
        self.generate(*data.body)?;
        self.buffer.push_str(&format!("\tb {}\n", start_label));
        self.buffer.push_str(&format!("{}:\n", end_label));
        Ok(())
    }

    fn gen_print(&mut self, expr: AstNode<'a>) -> Result<(), GeneratorError> {
        let expr_type = self.expr_type(&expr);

        self.generate(expr)?;

        let label_idx = self.label_count;
        self.label_count += 1;
        let fmt_label = format!(".L_fmt_{}", label_idx);

        let format_str = match expr_type {
            Type::String => "%s\\n",
            Type::Float => "%f\\n",
            _ => "%d\\n",
        };

        self.buffer.push_str("\t.section .rodata\n");
        self.buffer.push_str(&format!("{}:\n", fmt_label));
        self.buffer
            .push_str(&format!("\t.asciz \"{}\"\n", format_str));
        self.buffer.push_str("\t.text\n");

        if expr_type == Type::Float {
            self.buffer.push_str("\tfmov x1, d0\n");
        } else {
            self.buffer.push_str("\tmov x1, x0\n");
        }

        self.buffer.push_str(&format!("\tadrp x0, {}\n", fmt_label));
        self.buffer
            .push_str(&format!("\tadd x0, x0, :lo12:{}\n", fmt_label));

        self.buffer.push_str("\tsub sp, sp, #16\n");
        self.buffer.push_str("\tbl printf\n");
        self.buffer.push_str("\tadd sp, sp, #16\n");

        Ok(())
    }

    fn expr_type(&self, node: &AstNode<'a>) -> Type<'a> {
        match node {
            AstNode::IntLiteralExpr(_) => Type::Int,
            AstNode::BoolLiteralExpr(_) => Type::Bool,
            AstNode::FloatLiteralExpr(_) => Type::Float,
            AstNode::StringLiteralExpr(_) => Type::String,
            AstNode::VariableExpr(name) => self.local_vars.get(*name).unwrap().typ.clone(),
            AstNode::MemberAccessExpr(data) => {
                let obj_type = self.expr_type(&data.object);
                #[allow(clippy::collapsible_if)]
                if let Type::Struct(struct_name) = obj_type {
                    if let Some(fields) = self.struct_layouts.get(struct_name) {
                        if let Some((_, field_type)) = fields.get(&data.member) {
                            return field_type.clone();
                        }
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
                    _ => panic!("Index expression on non-array"),
                }
            }
            _ => Type::Int,
        }
    }

    fn write_to_file(&self, path: &str) -> io::Result<()> {
        fs::write(path, &self.buffer)
    }

    pub fn compile_executable(
        &self,
        input_file: &str,
        output_name: &str,
        keep_asm: bool,
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
        let s_path = output_dir.join(format!("{}.s", file_stem));
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

        if !keep_asm {
            let _ = fs::remove_file(&s_path);
        }

        if status.success() {
            Ok(())
        } else {
            Err(GeneratorError::GeneralError(
                "Compilation failed".to_string(),
            ))
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
