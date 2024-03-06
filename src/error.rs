use crate::value::Value;

/// Possible errors that may occur during execution.
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
    ExpectedColonAfterKey {
        line: usize,
    },
    
    // Environment errors.
    NameError {
        name: String,
        line: usize,
    },
    NotIndexable {
        line: usize,
    },
    OutOfBoundsIndex {
        index: usize,
        line: usize,
    },
    InsertNonStringIntoString {
        line: usize,
    },
    
    // Runtime errors.
    InvalidAssignmentTarget {
        line: usize,
    },
    ExpectedType {
        expected: String,
        got: String,
        line: usize,
    },
    NonNaturalIndex {
        got: Value,
        line: usize,
    },
    NonNumberIndex {
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
    LoopConditionNotBoolean {
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
    CannotHashDictionary {
        line: usize,
    },
    KeyError {
        key: Value,
        line: usize,
    },

    // Conversion
    CannotConvertToNumber {
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

/// Prints the error message for each error in `errors`.
pub fn report_errors(errors: &[ErrorType]) {
    println!("An error has occurred.");
    for error in errors {
        print_report(error);
    }
}

/// Prints the error message for an error.
fn print_report(error: &ErrorType) {
    match error {
        ErrorType::UnexpectedCharacter { character, line } => {
            println!("Line {}: unexpected character `{}`.", line, character);
        },
        ErrorType::UnterminatedString => {
            println!("Unterminated string at end of file.");
        },
        ErrorType::ExpectedCharacter { expected, line } => {
            println!("Line {}: expected character `{}`", line, expected);
        },
        ErrorType::ExpectedExpression { line } => {
            println!("Line {}: expected expression.", line);
        },
        ErrorType::ExpectedFunctionName { line } => {
            println!("Line {}: expected function name. Make sure it is not a keyword.", line);
        },
        ErrorType::ExpectedParameterName { line } => {
            println!("Line {}: expected parameter name after a comma in function declaration.", line);
        },
        ErrorType::ExpectedVariableName { line } => {
            println!("Line {}: expected variable name. Make sure it is not a keyword.", line);
        },
        ErrorType::ExpectedSemicolonAfterInit { line } => {
            println!("Line {}: expected `;` after initialising statement in `for` loop.", line);
        },
        ErrorType::ExpectedSemicolonAfterCondition { line } => {
            println!("Line {}: expected `;` after condition in `for` loop.", line);
        },
        ErrorType::ExpectedParenAfterIncrement { line } => {
            println!("Line {}: expected `)` after increment statement in `for` loop.", line);
        },
        ErrorType::ExpectedColonAfterKey { line } => {
            println!("Line {}: expected colon after dictionary key.", line);
        },
        ErrorType::NameError { ref name, line } => {
            println!("Line {}: `{}` is not defined.", line, name);
        },
        ErrorType::NotIndexable { line } => {
            println!("Line {}: the value is not indexable.", line);
        },
        ErrorType::OutOfBoundsIndex { index, line } => {
            println!("Line {}: index `{}` is out of bounds.", line, index);
        },
        ErrorType::InsertNonStringIntoString { line } => {
            println!("Line {}: attempted to insert a non-string into a string.", line);
        },
        ErrorType::InvalidAssignmentTarget { line } => {
            println!("Line {}: invalid assignment target.", line);
        },
        ErrorType::ExpectedType { ref expected, ref got, line } => {
            println!("Line {}: expected type {}; instead got type {}.", line, expected, got);
        },
        ErrorType::NonNaturalIndex { got, line } => {
            println!("Line {}: index evaluated to {}, which is not a positive integer.", line, got);
        },
        ErrorType::NonNumberIndex { got, line } => {
            println!("Line {}: index evaluated to a {}, which is not a positive integer.", line, got);
        },
        ErrorType::BinaryTypeError { ref expected, ref got_left, ref got_right, line } => {
            println!("Line {}: this operation requires both sides' types to be {}. Instead, got {} and {} respectively.", line, expected, got_left, got_right);
        },
        ErrorType::DivideByZero { line } => {
            println!("Line {}: divisor is 0.", line);
        },
        ErrorType::IfConditionNotBoolean { line } => {
            println!("Line {}: the `if` condition did not evaluate to a Boolean value.", line);
        },
        ErrorType::LoopConditionNotBoolean { line } => {
            println!("Line {}: the condition of the loop did not evaluate to a Boolean value.", line);
        },
        ErrorType::CannotCallName { line } => {
            println!("Line {}: cannot call name as a function.", line);
        },
        ErrorType::ArgParamNumberMismatch { arg_number, param_number, line } => {
            println!("Line {}: attempted to call function with {} argument(s), but function accepts {}.", line, arg_number, param_number);
        },
        ErrorType::CannotHashFunction { line } => {
            println!("Line {}: cannot hash function (functions cannot be used as keys in dictionary entries).", line);
        },
        ErrorType::CannotHashDictionary { line } => {
            println!("Line {}: cannot hash dictionary (dictionaries cannot be used as keys in dictionary entries).", line);
        },
        ErrorType::KeyError { key, line } => {
            println!("Line {}: key `{}` does not exist in the dictionary.", line, key);
        },
        ErrorType::CannotConvertToNumber { line } => {
            println!("Line {}: could not convert to a number.", line);
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
