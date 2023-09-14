#[derive(Clone, Debug, Eq, PartialEq)]
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
    NameNotIndexable {
        name: String,
        line: usize,
    },
    IndexError {
        name: String,
        index: usize,
        line: usize,
    },

    // Runtime errors.
    ExpectedTypeError {
        expected: String,
        got: String,
        line: usize,
    },
    TypeMismatchError {
        left: String,
        right: String,
        line: usize,
    },
    BinaryTypeError {
        expected: String,
        line: usize,
    },
    ExpressionNotIndexable {
        line: usize,
    },
    DivideByZero {
        line: usize,
    },
}

pub fn report(type_: &ErrorType) {
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
            println!("Line {0}: `{1}` is not defined.", &line.to_string(), name);
        },
        ErrorType::NameNotIndexable { ref name, line } => {
            println!("Line {0}: `{1}` is not indexable.", &line.to_string(), name);
        },
        ErrorType::IndexError { ref name, index, line } => {
            println!("Line {0}: index `{1}` is out of bounds for `{2}`.", &line.to_string(), index, name);
        },
        ErrorType::ExpectedTypeError { ref expected, ref got, line } => {
            println!("Expected type {0}, instead got type {1} on line {2}.", expected, got, &line.to_string());
        },
        ErrorType::TypeMismatchError { ref left, ref right, line } => {
            println!("Types for expression on line {0} are mismatched: left is {1}; right is {2}.", &line.to_string(), left, right);
        },
        ErrorType::BinaryTypeError { ref expected, line } => {
            println!("Line {0}: this operation requires both sides' types to be {1}.", &line.to_string(), expected);
        },
        ErrorType::ExpressionNotIndexable { line } => {
            println!("Line {0}: the expression is not indexable.", &line.to_string());
        },
        ErrorType::DivideByZero { line } => {
            println!("Divisor is 0 on line {0}", &line.to_string());
        },
    }
}
