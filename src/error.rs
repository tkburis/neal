#[derive(Clone, Debug, PartialEq)]
pub enum ErrorType {
    // Tokenizing errors.
    UnexpectedCharacter {
        character: char,
        line: usize,
    },
    UnterminatedString,

    // Parsing errors.
    ExpectedCharacter {
        expected: char,
        line: usize,
    },
    InvalidIndex {
        line: usize,
    },
    ExpectedExpression {
        line: usize,
    },
    ExpectedFunctionName {
        line: usize,
    },
    ExpectedParameterName {
        line: usize,
    },
    ExpectedVariableName {
        line: usize,
    },
    ExpectedSemicolonAfterInit {
        line: usize,
    },
    ExpectedSemicolonAfterCondition {
        line: usize,
    },
    ExpectedParenAfterIncrement {
        line: usize,
    },
    InvalidAssignmentTarget {
        line: usize,
    },

    // Environment errors.
    NameError {
        name: String,
        line: usize,
    },
}

pub fn report_and_return(type_: &ErrorType) {
    match type_ {
        ErrorType::UnexpectedCharacter { character, line } => {
            println!("Unexpected character `{0}` on line {1}.", &character.to_string(), &line.to_string());
        },
        ErrorType::UnterminatedString => {
            println!("Unterminated string at end of file.");
        },
        ErrorType::ExpectedCharacter { expected, line } => {
            println!("Expected character `{0}` on line {1}.", &expected.to_string(), &line.to_string());
        },
        ErrorType::InvalidIndex { line } => {
            println!("Invalid index on line {}. Make sure it is a positive integer.", &line.to_string());
        },
        ErrorType::ExpectedExpression { line } => {
            println!("Expected expression on line {}.", &line.to_string());
        },
        ErrorType::ExpectedFunctionName { line } => {
            println!("Expected function name on line {}. Make sure it is not a keyword.", &line.to_string());
        },
        ErrorType::ExpectedParameterName { line } => {
            println!("Expected parameter name in function declaration on line {}.", &line.to_string());
        },
        ErrorType::ExpectedVariableName { line } => {
            println!("Expected variable name in declaration on line {}.", &line.to_string());
        },
        ErrorType::ExpectedSemicolonAfterInit { line } => {
            println!("Expected `;` after initialising statement in `for` loop on line {}.", &line.to_string());
        },
        ErrorType::ExpectedSemicolonAfterCondition { line } => {
            println!("Expected `;` after condition in `for` loop on line {}.", &line.to_string());
        },
        ErrorType::ExpectedParenAfterIncrement { line } => {
            println!("Expected `)` after increment statement in `for` loop on line {}.", &line.to_string());
        },
        ErrorType::InvalidAssignmentTarget { line } => {
            println!("Invalid assignment target on line {}.", &line.to_string());
        },
        ErrorType::NameError { ref name, line } => {
            println!("Line {0}: '{1}' is not defined.", &line.to_string(), name);
        },
    }
}
