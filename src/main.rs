use std::{env, path::PathBuf, process::ExitCode};

mod abstractions;
mod ast;
mod error;
mod interpreter;
mod lexer;

fn main() -> ExitCode {
    let Some(source_path) = env::args().into_iter().nth(1) else {
        println!("Missing source code file path!");
        return ExitCode::FAILURE;
    };
    let lexer = lexer::Lexer::new(PathBuf::from(source_path));
    let tokens = lexer.get_tokens();
    let ast = match ast::Parser::parse_program(tokens) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parsing error: {:?}", e);
            return ExitCode::FAILURE;
        }
    };

    let mut interpreter = interpreter::Interpreter::new();

    if let Err(err) = interpreter.evaluate_program(&ast) {
        eprintln!("Interpretation error: {:?}", err);
        return ExitCode::FAILURE;
    }
    return ExitCode::SUCCESS;
}