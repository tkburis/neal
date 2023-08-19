#[derive(Clone, Debug, PartialEq)]
pub enum ErrorType {
    UnexpectedCharacter {
        character: char,
        line: usize,
    },
    UnterminatedString,
    ExpectedCharacter {
        expected: String,
        line: usize,
    },
    InvalidIndex {
        line: usize,
    },
    ExpectedExpression {
        line: usize,
    },
}

pub fn report_and_return(type_: ErrorType) -> ErrorType {
    match type_ {
        ErrorType::UnexpectedCharacter { character, line } => {
            println!("Unexpected character `{0}` on line {1}.", &character.to_string(), &line.to_string());
        },
        ErrorType::UnterminatedString => {
            println!("Unterminated string at end of file.");
        },
        ErrorType::ExpectedCharacter { ref expected, line } => {
            println!("Expected character `{0}` on line {1}.", expected, &line.to_string());
        },
        ErrorType::InvalidIndex { line } => {
            println!("Invalid index on line {}. Make sure it is a positive integer.", &line.to_string())
        },
        ErrorType::ExpectedExpression { line } => {
            println!("Expected expression on line {}.", &line.to_string())
        }
    }
    type_
}
