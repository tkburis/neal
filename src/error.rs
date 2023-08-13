#[derive(Debug, PartialEq)]
pub enum ErrorType {
    UnexpectedCharacter {
        character: char,
        line: usize,
    },
    UnterminatedString,
}

pub fn report(type_: ErrorType) -> ErrorType {
    match type_ {
        ErrorType::UnexpectedCharacter { character, line } => {
            println!("Unexpected character: {0} on line {1}.", &character.to_string(), &line.to_string())
        },
        ErrorType::UnterminatedString => {
            println!("Unterminated string at end of file.")
        }
    }
    type_
}
