use crate::code_generator::Generator;
use crate::parser::{Parser, ParserError};
use crate::scanner::Scanner;
use crate::semantic_analyzer::Analyzer;
use std::env;
use std::fs;
use std::process;

mod code_generator;
mod parser;
mod scanner;
mod semantic_analyzer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <source_file> [-o <output_file>]", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];

    let mut output_file = "a.out";
    let mut i = 2;
    while i < args.len() {
        if args[i] == "-o" && i + 1 < args.len() {
            output_file = &args[i + 1];
            i += 2;
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
            eprintln!("[ERROR][{}:{}]: {}", err.line, err.column, err.message);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(&tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            let error_message = match err {
                ParserError::UnexpectedToken { expected, found } => {
                    format!("[ERROR]: Expected {}, but found {}", expected, found)
                }
                ParserError::UnexpectedEoF => String::from("No valid file end was found"),
                other => {
                    format!("[ERROR]: {:?}", other)
                }
            };

            eprintln!("{}", error_message);
            std::process::exit(1);
        }
    };

    let mut analyzer = Analyzer::new();
    match analyzer.analyze(ast.clone()) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("[ERROR]: {:?}", err);
            std::process::exit(1);
        }
    }

    let mut generator = Generator::new();
    match generator.generate(ast) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("[ERROR]: {:?}", err);
            std::process::exit(1);
        }
    }

    match generator.compile_executable(input_file, output_file) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("[ERROR]: {:?}", err);
            std::process::exit(1);
        }
    }
}
