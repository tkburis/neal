mod environment;
mod error;
mod expr;
mod hash_table;
mod interpreter;
mod parser;
mod stmt;
mod token;
mod tokenizer;
mod value;

use std::env;
use std::io;
use std::io::Write;
use std::process;
use std::fs;

use parser::Parser;
use tokenizer::Tokenizer;
use interpreter::Interpreter;

/// Driver code.
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 { 
        // Only two arguments are expected. Note that the args[0] will be the name of the binary.
        eprintln!("Usage: cargo run [-- script]");
        process::exit(64);
    } else if args.len() == 2 {
        // The second argument will be the file path of the source code.
        run_file(&args[1]);
    } else {
        // No file path was given. In this case, we run the REPL.
        run_repl();
    }
}

/// Runs the source code given at the file path.
fn run_file(file_path: &str) {
    let source = fs::read_to_string(file_path).expect("Failed to read file.");
    let mut interpreter = Interpreter::new();  // An Interpreter object has to be provided to `run()`, as explained below.
    
    run(&source, &mut interpreter);
}

/// Runs the interactive REPL in the console.
fn run_repl() {
    // We need the same Interpreter instance across all REPL command to preserve the environment (variables, etc.).
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        io::stdout().flush().expect("Error: flush failed");  // to flush out "> "
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");
        run(&line, &mut interpreter);
    }
}

fn run(source: &str, interpreter: &mut Interpreter) {
    let mut tokenizer = Tokenizer::new(source);
    let Ok(tokens) = tokenizer.tokenize() else {
        // If `tokenize()` did not return `Ok(tokens)`, i.e., there was an error, we terminate execution.
        return;
    };

    let mut parser = Parser::new(tokens);
    let Ok(ast) = parser.parse() else {
        // Similarly, if `parse()` did not return `Ok(ast)`, i.e., there was an error, we terminate execution.
        return;
    };

    interpreter.interpret(ast);
}
