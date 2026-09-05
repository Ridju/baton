use code_generator::Generator;
use parser::{Parser, ParserError};
use scanner::Scanner;
use semantic_analyzer::Analyzer;
use std::env;
use std::fs;
use std::process;

fn print_error_with_snippet(source: &str, line: usize, column: usize, message: &str) {
    eprintln!("\x1b[1;31m[ERROR]\x1b[0m: {}", message);
    eprintln!("  --> line {}:{}", line, column);

    let lines: Vec<&str> = source.lines().collect();
    if line > 0 && line <= lines.len() {
        let error_line = lines[line - 1];
        eprintln!("   |");
        eprintln!("{:3} | {}", line, error_line);

        let padding = " ".repeat(column.saturating_sub(1));
        eprintln!("   | {}\x1b[1;31m^\x1b[0m", padding);
    }
    eprintln!();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: {} <source_file> [-o <output_file>] [--keep-asm]",
            args[0]
        );
        process::exit(1);
    }

    let input_file = &args[1];

    let mut output_file = "a.out";
    let mut keep_asm = false;
    let mut i = 2;

    while i < args.len() {
        if args[i] == "-o" && i + 1 < args.len() {
            output_file = &args[i + 1];
            i += 2;
        } else if args[i] == "--keep-asm" || args[i] == "-k" {
            keep_asm = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    let source = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error: Could not read file '{}': {}", input_file, err);
            process::exit(1);
        }
    };

    let mut sc = Scanner::new(&source);
    let tokens = match sc.scan_source() {
        Ok(tokens) => tokens,
        Err(err) => {
            print_error_with_snippet(&source, err.line, err.column, &err.message);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(&tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            let (line, column, msg) = match err {
                ParserError::UnexpectedToken {
                    expected,
                    found,
                    line,
                    column,
                } => (
                    line,
                    column,
                    format!("Expected {:?}, but found {:?}", expected, found),
                ),
                ParserError::UnexpectedEoF => (0, 0, String::from("No valid file end was found")),
                other => (0, 0, format!("{:?}", other)),
            };
            print_error_with_snippet(&source, line, column, &msg);
            std::process::exit(1);
        }
    };

    let mut analyzer = Analyzer::new();
    if let Err(err) = analyzer.analyze(&ast) {
        let (line, column, msg) = match &err {
            semantic_analyzer::SemanticError::Redefinition {
                line,
                column,
                message,
                ..
            } => (*line, *column, message.clone()),
            semantic_analyzer::SemanticError::UndefinedVariable {
                line, column, name, ..
            } => (*line, *column, format!("Undefined variable '{}'", name)),
            semantic_analyzer::SemanticError::UndefinedFunction {
                line, column, name, ..
            } => (*line, *column, format!("Undefined function '{}'", name)),
            semantic_analyzer::SemanticError::UndefinedStruct {
                line, column, name, ..
            } => (*line, *column, format!("Undefined struct '{}'", name)),
            semantic_analyzer::SemanticError::TypeMismatch {
                line,
                column,
                message,
                ..
            } => (*line, *column, message.clone()),
            semantic_analyzer::SemanticError::InvalidReturnType {
                line,
                column,
                message,
                ..
            } => (*line, *column, message.clone()),
            semantic_analyzer::SemanticError::NotAncillaryElement {
                line,
                column,
                name,
                expected,
            } => (*line, *column, format!("'{}' is not a {}", name, expected)),
            semantic_analyzer::SemanticError::NotImplemented {
                line,
                column,
                message,
            } => (*line, *column, message.clone()),
        };
        print_error_with_snippet(&source, line, column, &msg);
        std::process::exit(1);
    }

    let mut generator = Generator::new();
    if let Err(err) = generator.generate(ast) {
        eprintln!("[ERROR]: {:?}", err);
        std::process::exit(1);
    }

    if let Err(err) = generator.compile_executable(input_file, output_file, keep_asm) {
        eprintln!("[ERROR]: {:?}", err);
        std::process::exit(1);
    }
}
