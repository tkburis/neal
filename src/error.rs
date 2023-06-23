pub enum ErrorType {
    UnexpectedEof,
    UnexpectedCharacter {
        character: char,
        line: usize,
    },
    UnterminatedString,
}

pub fn report(type_: ErrorType) {
    match type_ {
        ErrorType::UnexpectedEof => {
            println!("Unexpected end of file.")
        },
        ErrorType::UnexpectedCharacter { character, line } => {
            println!("Unexpected character: {0} on line {1}.", &character.to_string(), &line.to_string())
        },
        ErrorType::UnterminatedString => {
            println!("Unterminated string.")
        }
    }
}
