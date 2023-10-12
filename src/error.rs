use crate::value::Value;

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
    ExpectedColonAfterKey {
        line: usize,
    },
    
    // Environment errors.
    NameError {
        name: String,
        line: usize,
    },
    NotIndexableError {  // TODO: Maybe no need for `name`.
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
        got: Value,
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
    CannotCallName {
        line: usize,
    },
    ArgParamNumberMismatch {
        arg_number: usize,
        param_number: usize,
        line: usize,
    },

    // Hash table
    CannotHashFunction {
        line: usize,
    },
    KeyError {
        key: Value,
        line: usize,
    },

    // Conversion
    ConvertToNumberError {
        line: usize,
    },

    // Misc.
    ThrownReturn {
        value: Value,
        line: usize,
    },
    ThrownBreak {
        line: usize,
    },
    ThrownLiteralAssignment {
        line: usize,
    },
}

pub fn report_errors(errors: &[ErrorType]) {
    println!("An error has occurred.");
    for error in errors {
        print_report(error);
    }
}

fn print_report(error: &ErrorType) {
    match error {
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
        ErrorType::ExpectedColonAfterKey { line } => {
            println!("Line {}: expected colon after dictionary key.", line);
        },
        ErrorType::NameError { ref name, line } => {
            println!("Line {0}: `{1}` is not defined.", line, name);
        },
        ErrorType::NotIndexableError { name, line } => {
            if let Some(n) = name {
                println!("Line {0}: `{1}` is not indexable.", line, n);
            } else {
                println!("Line {}: the value is not indexable.", line);
            }
        },
        ErrorType::OutOfBoundsIndexError { name, index, line } => {
            if let Some(n) = name {
                println!("Line {0}: index `{1}` is out of bounds for `{2}`.", line, index, n);
            } else {
                println!("Line {0}: index `{1}` is out of bounds.", line, index)
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
        ErrorType::CannotCallName { line } => {
            println!("Line {}: cannot call name as a function.", line);
        },
        ErrorType::ArgParamNumberMismatch { arg_number, param_number, line } => {
            println!("Line {}: attempted to call function with {} argument(s), but function accepts {}", line, arg_number, param_number);
        },
        ErrorType::CannotHashFunction { line } => {
            println!("Line {}: cannot hash `Function` type.", line);
        },
        ErrorType::KeyError { key, line } => {
            println!("Line {}: key `{}` does not exist in the dictionary.", line, key);
        },
        ErrorType::ConvertToNumberError { line } => {
            println!("Line {}: could not convert to Number.", line);
        },

        ErrorType::ThrownReturn { value: _ , line} => {
            println!("Line {}: `return` has to be used within a function.", line);
        },
        ErrorType::ThrownBreak { line } => {
            println!("Line {}: `break` has to be used within a loop.", line);
        },
        ErrorType::ThrownLiteralAssignment { line } => {
            println!("Line {}: attempt to assign to a literal.", line);
        },
    }
}
