// use crate::hash_table::HashTable;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen,
    LeftCurly, RightCurly,
    LeftSquare, RightSquare,
    Colon, Comma, Dot, Minus, Percent,
    Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier, String_, Number,

    // Keywords.
    And, Break, Class, Else, False,
    Func, For, If, Null, Or, Print,
    Return, Self_, True, Var, While,

    Eof,
}

/// `Literal` represents 'front-end' values from the source code.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Number(f64),
    String_(String),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub type_: TokenType,  // Type of token.
    pub lexeme: String,  // The 'original' from the source code.
    pub literal: Literal,  // The literal value (number/string/null if N/A) the token represents.
    pub line: usize,  // The line number the token was present in.
}
