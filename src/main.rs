mod environment;
mod error;
mod expr;
mod interpreter;
mod parser;
mod stmt;
mod token;
mod tokenizer;

use std::env;
use std::io;
use std::io::Write;
use std::process;
use std::fs;

use error::ErrorType;
use parser::Parser;
use tokenizer::Tokenizer;

use crate::interpreter::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: cargo run [-- script]");
        process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(file_path: &str) {
    let source = fs::read_to_string(file_path).expect("Failed to read file.");
}

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().expect("Flush failed");  // to flush out "> "
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");
        let _ = run(&line);
    }
}

fn run(source: &str) {
    let mut tokenizer = Tokenizer::new(source);
    let Ok(tokens) = tokenizer.tokenize() else {
        return;
    };
    println!("TOKENS: {:?}", tokens);

    let mut parser = Parser::new(tokens);
    let Ok(ast) = parser.parse() else {
        return;
    };
    println!("AST: {:?}", ast);

    let mut interpreter = Interpreter::new();
    interpreter.interpret(ast);
}
