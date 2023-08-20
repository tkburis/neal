use crate::token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Array {
        elements: Vec<Expr>,
    },
    Assignment {
        // Assignment is an expression for two reasons:
        // 1. Expressions like `a = b = 5` work;
        // 2. Calls have to be checked *before* assignments.
        // Otherwise, the statement `f()` will not work because we expect an `=` after the identifier.
        // Note `target` is an expression to allow both `Variable`s and `Element`s.
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,  // `Expr` so that f()() works
        arguments: Vec<Expr>,
    },
    Element {  // ? Maybe this should be combined with `Variable`...
        array: Box<Expr>,  // Should resolve to `Array`.
        index: usize,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: token::Literal,
    },
    Unary {
        operator: token::Token,  // Either `!` or `-`.
        right: Box<Expr>,
    },
    Variable {
        name: String,  // More specifically, `Token` with `Identifier` type. Let's try String. Formerly Token.
    },
}
