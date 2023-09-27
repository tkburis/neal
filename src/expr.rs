use crate::{token, hash_table::KeyValue};

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub line: usize,
    pub expr_type: ExprType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprType {
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
    Dictionary {
        elements: Vec<KeyValue<Expr>>,
    },
    Element {  // ? Maybe this should be combined with `Variable`...
        // ! This should probably NOT be combined with `Variable` because e.g., [1,2,3][0] has to evaluate [1,2,3] first.
        array: Box<Expr>,  // Should resolve to `Array` or `Variable`.
        index: Box<Expr>,
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
