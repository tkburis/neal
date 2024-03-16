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

use std::{env, io, io::Write, fs};

use parser::Parser;
use tokenizer::Tokenizer;
use interpreter::Interpreter;

/// Driver code.
fn main() {
    let args: Vec<String> = env::args().collect();

    // Note that `args[0]` will be the name of the binary.
    // So to check whether one argument has been passed, we check if `args.len() == 2`.
    if args.len() > 2 {
        // Only one given argument is expected.
        eprintln!("Usage: nea.exe [script]");
    } else if args.len() == 2 {
        // `args[1]` will be the given argument, i.e., the file path of the source code.
        run_file(&args[1]);
    } else {
        // No arguments were given. In this case, we run the REPL interface.
        run_repl();
    }
}

/// Runs the source code given at the file path.
fn run_file(file_path: &str) {
    // Reading from the file path. If an error occurs, the `expect()` method will print "Failed to read file." and terminate execution.
    let source = fs::read_to_string(file_path).expect("Failed to read file.");

    // An Interpreter object has to be provided to `run()`, as explained below.
    let mut interpreter = Interpreter::new();
    
    run(&source, &mut interpreter);
}

/// Runs the interactive REPL interfae in the console.
fn run_repl() {
    // We need the same `Interpreter` instance across all REPL source code inputs to preserve the variables and functions stored in the environment.
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        io::stdout().flush().expect("Error: flush failed");  // to flush out "> "

        // Read user input into `line`.
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        run(&line, &mut interpreter);
    }
}

/// Executes the source code string with the given interpreter instance.
fn run(source: &str, interpreter: &mut Interpreter) {
    // Lexical analysis.
    let mut tokenizer = Tokenizer::new(source);
    // If the source code was tokenized without errors, assign the token sequence to `tokens`.
    let Ok(tokens) = tokenizer.tokenize() else {
        // If an error occurred, stop trying to execute the current source code string.
        // If the user is using a REPL interface, this does not then end the session but simply prompts the user for a new source code input, as expected.
        return;
    };

    // Syntax analysis.
    let mut parser = Parser::new(tokens);
    // Similarly, if the token sequence was parsed without errors, assign the abstract syntax tree to `ast`.
    let Ok(ast) = parser.parse() else {
        // If an error occurred, stop trying to execute the current source code string.
        return;
    };

    // Evaluation and execution.
    interpreter.interpret(ast);
}
