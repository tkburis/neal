// Possible types of tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
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
    And, Break, Else, False,
    Func, For, If, Null, Or, Print,
    Return, True, Var, While,

    Eof,
}

/// Literal values present in the source code.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Number(f64),
    String_(String),
    Bool(bool),
    Null,
}

/// A token.
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub type_: TokenType,  // Type of token.
    pub lexeme: String,  // The substring from the source code from which the token was constructed.
    pub literal: Literal,  // The literal value (number/string/Boolean) the token represents; otherwise, Literal::Null.
    pub line: usize,  // The line number of the source code from which the token was constructed.
}
