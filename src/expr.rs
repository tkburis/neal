use crate::token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Array {
        elements: Vec<Expr>,
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
    }
}
