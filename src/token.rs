/// Possible types of tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen,
    LeftCurly, RightCurly,
    LeftSquare, RightSquare,
    Colon, Comma, Minus, Percent,
    Plus, Semicolon, Slash, Star,

    // One- or two-character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    True, False, String_, Number,

    // Keywords.
    And, Break, Else,
    Func, For, If, Null, Or, Print,
    Return, Var, While,

    Identifier, Eof
}

/// Literal values declared in the source code.
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
    pub type_: TokenType,  // The type of the token.
    pub lexeme: String,  // The source code substring from which the token was constructed.
    pub literal: Literal,  // The literal value (number/string/Boolean) the token represents; if the token is not a literal, will be set to the `Null` variant.
    pub line: usize,  // The line number of the source code from which the token was constructed.
}
