use crate::parser::AstNode;
use std::fs;
use std::io;
use std::process::Command;

#[derive(Debug, PartialEq)]
enum GeneratorError {
    GeneralError(String),
}

#[derive(Debug)]
struct Generator {
    buffer: String,
}

impl Generator {
    fn new() -> Generator {
        Generator {
            buffer: String::new(),
        }
    }

    fn generate(&mut self, ast: AstNode) -> Result<(), GeneratorError> {
        match ast {
            AstNode::Programm(nodes) => {
                self.buffer.push_str(".global _main\n.text\n\n");
                for node in nodes {
                    self.generate(node)?
                }
                Ok(())
            }
            AstNode::FunctionDecl(func_data) => {
                self.buffer.push_str(&format!("_{}:\n", func_data.name));
                self.buffer
                    .push_str("\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n");
                self.generate(*func_data.body)?;
                Ok(())
            }
            AstNode::ReturnStatement(node) => {
                self.generate(*node)?;
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
        }
    }

    fn write_to_file(&self, path: &str) -> io::Result<()> {
        fs::write(path, &self.buffer)
    }

    pub fn compile_executable(
        &self,
        s_filename: &str,
        output_name: &str,
    ) -> Result<(), GeneratorError> {
        self.write_to_file(s_filename)
            .map_err(|e| GeneratorError::GeneralError(format!("Could not write file: {}", e)))?;

        let status = Command::new("cc")
            .arg(s_filename)
            .arg("-o")
            .arg(output_name)
            .status()
            .map_err(|e| {
                GeneratorError::GeneralError(format!("Compiler could not be started: {}", e))
            })?;

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
    use crate::parser::{AstNode, FunctionDeclData, Type};

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

        let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tmov x0, #42\n\tldp x29, x30, [sp], #16\n\tret\n";

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

        let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tmov x0, #0\n\tldp x29, x30, [sp], #16\n\tret\n";

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

        let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tmov x0, #0\n\tldp x29, x30, [sp], #16\n\tret\n_helper_func:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tmov x0, #100\n\tldp x29, x30, [sp], #16\n\tret\n";

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

        let expected_assembly = ".global _main\n.text\n\n_main:\n\tstp x29, x30, [sp, #-16]!\n\tmov x29, sp\n\tmov x0, #5\n\tmov x0, #10\n\tldp x29, x30, [sp], #16\n\tret\n";

        assert_eq!(generator.buffer, expected_assembly);
    }
}
