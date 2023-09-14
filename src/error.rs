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
    NotIndexableError {
        name: Option<String>,
        line: usize,
    },
    OutOfBoundsIndexError {
        name: Option<String>,
        index: usize,
        line: usize,
    },
    
    // Runtime errors.
    ExpectedTypeError {
        expected: String,
        got: String,
        line: usize,
    },
    NonNaturalIndexError {
        got: crate::token::Value,
        line: usize,
    },
    NonNumberIndexError {
        got: String,
        line: usize,
    },
    BinaryTypeError {
        expected: String,
        got_left: String,
        got_right: String,
        line: usize,
    },
    DivideByZero {
        line: usize,
    },
    IfConditionNotBoolean {
        line: usize,
    },
    WhileConditionNotBoolean {
        line: usize,
    },
}

pub fn report(type_: &ErrorType) {
    match type_ {
        ErrorType::UnexpectedCharacter { character, line } => {
            println!("Line {0}: unexpected character `{1}`.", line, character);
        },
        ErrorType::UnterminatedString => {
            println!("Unterminated string at end of file.");
        },
        ErrorType::ExpectedCharacter { expected, line } => {
            println!("Line {0}: expected character `{1}`", line, expected);
        },
        ErrorType::ExpectedExpression { line } => {
            println!("Line {}: expected expression.", line);
        },
        ErrorType::ExpectedFunctionName { line } => {
            println!("Line {}: expected function name. Make sure it is not a keyword.", line);
        },
        ErrorType::ExpectedParameterName { line } => {
            println!("Line {}: expected parameter name in function declaration.", line);
        },
        ErrorType::ExpectedVariableName { line } => {
            println!("Line {}: expected variable name in declaration", line);
        },
        ErrorType::ExpectedSemicolonAfterInit { line } => {
            println!("Line {}: expected `;` after initialising statement in `for` loop", line);
        },
        ErrorType::ExpectedSemicolonAfterCondition { line } => {
            println!("Line {}: expected `;` after condition in `for` loop", line);
        },
        ErrorType::ExpectedParenAfterIncrement { line } => {
            println!("Line {}: expected `)` after increment statement in `for` loop.", line);
        },
        ErrorType::InvalidAssignmentTarget { line } => {
            println!("Line {}: invalid assignment target", line);
        },
        ErrorType::NameError { ref name, line } => {
            println!("Line {0}: `{1}` is not defined.", line, name);
        },
        ErrorType::NotIndexableError { name, line } => {
            if let Some(n) = name {
                println!("Line {0}: `{1}` is not indexable.", line, n);
            } else {
                println!("Line {}: the expression is not indexable.", line);
            }
        },
        ErrorType::OutOfBoundsIndexError { name, index, line } => {
            if let Some(n) = name {
                println!("Line {0}: index `{1}` is out of bounds for `{2}`.", line, index, n);
            } else {
                println!("Line {0}: index `{1}` is out of bounds for array.", line, index)
            }
        },
        ErrorType::ExpectedTypeError { ref expected, ref got, line } => {
            println!("Line {0}: expected type {1}, instead got type {2}", line, expected, got);
        },
        ErrorType::NonNaturalIndexError { got, line } => {
            println!("Line {0}: index evaluated to {1}, which is not a positive integer.", line, got);
        },
        ErrorType::NonNumberIndexError { got, line } => {
            println!("Line {0}: index evaluated to a {1}, which is not a positive integer.", line, got);
        },
        ErrorType::BinaryTypeError { ref expected, ref got_left, ref got_right, line } => {
            println!("Line {0}: this operation requires both sides' types to be {1}. Instead, got {2} and {3} respectively.", line, expected, got_left, got_right);
        },
        ErrorType::DivideByZero { line } => {
            println!("Line {}: divisor is 0", line);
        },
        ErrorType::IfConditionNotBoolean { line } => {
            println!("Line {}: the `if` condition does not evaluate to a Boolean value.", line);
        },
        ErrorType::WhileConditionNotBoolean { line } => {
            println!("Line {}: the condition of the `while` loop does not evaluate to a Boolean value.", line);
        },
    }
}
