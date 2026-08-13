use crate::parser::{AstNode, BinaryOperator};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq)]
pub enum GeneratorError {
    GeneralError(String),
}

#[derive(Debug)]
pub struct Generator {
    buffer: String,
    local_vars: HashMap<String, i32>,
    stack_offset: i32,
    label_count: usize,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            buffer: String::new(),
            local_vars: HashMap::new(),
            stack_offset: -8,
            label_count: 0,
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
            AstNode::BlockStatement(nodes) => {
                for node in nodes {
                    self.generate(node)?;
                }
                Ok(())
            }
            AstNode::BinaryExpr(bin_data) => {
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
                Ok(())
            }
            AstNode::VarDeclStatement(var_data) => {
                if let Some(init) = var_data.initializer {
                    self.generate(*init)?;
                } else {
                    self.buffer.push_str("\tmov x0, #0\n");
                }

                let offset = self.stack_offset;
                self.local_vars.insert(var_data.name.clone(), offset);
                self.stack_offset -= 8;

                self.buffer
                    .push_str(&format!("\tstr x0, [x29, #{}]\n", offset));
                Ok(())
            }
            AstNode::AssignmentStatement(assign_data) => {
                self.generate(*assign_data.value)?;

                let offset = self.local_vars.get(&assign_data.name).unwrap_or_else(|| {
                    panic!(
                        "Undefined variable '{}' in code generator",
                        assign_data.name
                    );
                });
                self.buffer
                    .push_str(&format!("\tstr x0, [x29, #{}]\n", offset));
                Ok(())
            }
            AstNode::VariableExpr(name) => {
                let offset = self.local_vars.get(&name).unwrap_or_else(|| {
                    panic!("Undefined variable '{}' in code generator", name);
                });
                self.buffer
                    .push_str(&format!("\tldr x0, [x29, #{}]\n", offset));
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
    use crate::parser::{AstNode, BinaryExpData, FunctionDeclData, Type};

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
}
