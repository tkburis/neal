mod token;
mod scanner;
mod error;

use core::ascii;
use std::env;
use std::io;
use std::io::Write;
use std::process;
use std::fs;

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
        run(&line);
    }
}

fn run(source: &str) -> Result<(), ()> {
    let mut scanner = scanner::Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    println!("{:?}", tokens);
    Ok(())
}
