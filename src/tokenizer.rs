use crate::{token::{Token, TokenType, Literal}, error::{self, ErrorType}};

/// The states of the DFA.
#[derive(Debug)]
enum State {
    Start,
    GotLeftParen,
    GotRightParen,
    GotLeftCurly,
    GotRightCurly,
    GotLeftSquare,
    GotRightSquare,
    GotColon,
    GotComma,
    GotMinus,
    GotPercent,
    GotPlus,
    GotSemicolon,
    GotSlash,
    GotStar,
    InComment,
    GotBang,
    GotBangEqual,
    GotEqual,
    GotEqualEqual,
    GotGreater,
    GotGreaterEqual,
    GotLess,
    GotLessEqual,
    InStringDouble,
    InStringSingle,
    GotString,
    InNumberBeforeDot,
    InNumberAfterDot,
    InWord,  // Identifiers and keywords.
    NoOp,
}

/// Performs lexical analysis.
pub struct Tokenizer<'a> {
    source: &'a str,  // Source code.
    tokens: Vec<Token>,  // Tokens that have been tokenized from source code.
    start: usize,  // Points to the start of the current token.
    current_index: usize,  // Points to the *next* character to be scanned.
    current_line: usize,  // Keeps track of the current line number.
}

impl<'a> Tokenizer<'a> {
    /// Initialise a new instance of `Tokenizer`, given the string source code.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current_index: 0,
            current_line: 1,
        }
    }

    /// Returns the sequence of tokens from the source code.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, ErrorType> {
        while self.source.chars().nth(self.current_index).is_some() {
            // If current_index has not reached the end of the source code, scan the next token.
            match self.scan_token() {
                Ok(token_opt) => {
                    // It is possible that `scan_token()` returns Ok(None) if the DFA lands on the NoOp state.
                    if let Some(token) = token_opt {
                        self.tokens.push(token);
                    }
                },
                Err(error) => {
                    // If an error has occurred during the `scan_token()` call, report the error and terminate execution.
                    error::report_errors(&[error.clone()]);
                    return Err(error);  // Return Err() so that the driver code knows to end execution.
                }
            }
        }

        // Push an EOF token to end the token sequence.
        self.tokens.push(Token {
            type_: TokenType::Eof,
            lexeme: String::from(""),
            literal: Literal::Null,
            line: self.current_line
        });

        Ok(self.tokens.clone())
    }

    /// Scans the token starting from `current_index`.
    fn scan_token(&mut self) -> Result<Option<Token>, ErrorType> {
        let mut current_state = State::Start;  // The current state of the finite automaton.
        
        loop {
            // It is possible that the tokenizer reaches the end of the source code before `scan_token()` returns.
            // So we account for `current_char_opt` being None in all possible current states.
            let current_char_opt = self.source.chars().nth(self.current_index);

            match current_state {
                State::Start => {
                    self.start = self.current_index;  // The next token starts here.
                    if let Some(current_char) = current_char_opt {
                        match current_char {
                            '(' => current_state = State::GotLeftParen,
                            ')' => current_state = State::GotRightParen,
                            '{' => current_state = State::GotLeftCurly,
                            '}' => current_state = State::GotRightCurly,
                            '[' => current_state = State::GotLeftSquare,
                            ']' => current_state = State::GotRightSquare,
                            ':' => current_state = State::GotColon,
                            ',' => current_state = State::GotComma,
                            '-' => current_state = State::GotMinus,
                            '%' => current_state = State::GotPercent,
                            '+' => current_state = State::GotPlus,
                            ';' => current_state = State::GotSemicolon,
                            '/' => current_state = State::GotSlash,
                            '*' => current_state = State::GotStar,
                            
                            // For these tokens, we have to see the next character to be able to correctly identify the token.
                            '!' => current_state = State::GotBang,
                            '=' => current_state = State::GotEqual,
                            '>' => current_state = State::GotGreater,
                            '<' => current_state = State::GotLess,
                            
                            // Literals.
                            '"' => current_state = State::InStringDouble,
                            '\'' => current_state = State::InStringSingle,
                            
                            '0'..='9' => current_state = State::InNumberBeforeDot,
                            
                            // Identifiers and keywords.
                            'a'..='z' | 'A'..='Z' | '_' => current_state = State::InWord,
    
                            // Comments.
                            '#' => current_state = State::InComment,

                            // Whitespace.
                            ' ' | '\r' | '\t' => current_state = State::NoOp,
    
                            '\n' => {
                                self.current_line += 1;
                                current_state = State::NoOp;
                            },
    
                            other => {
                                // If the character does not match any of the above rules, raise an UnexpectedCharacter error.
                                return Err(ErrorType::UnexpectedCharacter {
                                    character: other,
                                    line: self.current_line,
                                });
                            },
                        }
                    } else {
                        // Note that this should be unreachable; the Start state is only ever accessed at the first iteration of the loop,
                        // and we have already checked we are not at the end in `tokenize()`.
                        return Ok(None);
                    }
                },

                State::GotLeftParen => return Ok(Some(self.construct_token(TokenType::LeftParen))),
                State::GotRightParen => return Ok(Some(self.construct_token(TokenType::RightParen))),
                State::GotLeftCurly => return Ok(Some(self.construct_token(TokenType::LeftCurly))),
                State::GotRightCurly => return Ok(Some(self.construct_token(TokenType::RightCurly))),
                State::GotLeftSquare => return Ok(Some(self.construct_token(TokenType::LeftSquare))),
                State::GotRightSquare => return Ok(Some(self.construct_token(TokenType::RightSquare))),
                State::GotColon => return Ok(Some(self.construct_token(TokenType::Colon))),
                State::GotComma => return Ok(Some(self.construct_token(TokenType::Comma))),
                State::GotMinus => return Ok(Some(self.construct_token(TokenType::Minus))),
                State::GotPercent => return Ok(Some(self.construct_token(TokenType::Percent))),
                State::GotPlus => return Ok(Some(self.construct_token(TokenType::Plus))),
                State::GotSemicolon => return Ok(Some(self.construct_token(TokenType::Semicolon))),
                State::GotSlash => return Ok(Some(self.construct_token(TokenType::Slash))),
                State::GotStar => return Ok(Some(self.construct_token(TokenType::Star))),
                
                State::GotBang => {
                    if current_char_opt == Some('=') {
                        current_state = State::GotBangEqual;
                    } else {
                        // If the character isn't `=` or we are at the end, just make a `Bang` token.
                        return Ok(Some(self.construct_token(TokenType::Bang)));
                    }
                },

                State::GotEqual => {
                    if current_char_opt == Some('=') {
                        current_state = State::GotEqualEqual;
                    } else {
                        return Ok(Some(self.construct_token(TokenType::Equal)));
                    }
                },
                State::GotGreater => {
                    if current_char_opt == Some('=') {
                        current_state = State::GotGreaterEqual;
                    } else {
                        return Ok(Some(self.construct_token(TokenType::Greater)));
                    }
                },
                State::GotLess => {
                    if current_char_opt == Some('=') {
                        current_state = State::GotLessEqual;
                    } else {
                        return Ok(Some(self.construct_token(TokenType::Less)));
                    }
                },
                
                State::GotBangEqual => return Ok(Some(self.construct_token(TokenType::BangEqual))),
                State::GotEqualEqual => return Ok(Some(self.construct_token(TokenType::EqualEqual))),
                State::GotGreaterEqual => return Ok(Some(self.construct_token(TokenType::GreaterEqual))),
                State::GotLessEqual => return Ok(Some(self.construct_token(TokenType::LessEqual))),
                
                State::InStringDouble => {
                    if current_char_opt == Some('"') {
                        current_state = State::GotString;
                    } else if current_char_opt.is_none() {
                        // We have reached the end and there was no closing `"`.
                        return Err(ErrorType::UnterminatedString);
                    }
                },
                State::InStringSingle => {
                    if current_char_opt == Some('\'') {
                        current_state = State::GotString;
                    } else if current_char_opt.is_none() {
                        // We have reached the end and there was no closing `"`.
                        return Err(ErrorType::UnterminatedString);
                    }
                },
                State::GotString => {
                    return Ok(Some(self.construct_token_with_literal(
                        TokenType::String_,
                        Literal::String_(self.source[self.start+1..self.current_index-1].to_owned())
                    )));
                },

                State::InNumberBeforeDot => {
                    match current_char_opt {
                        Some(current_char) => {
                            if current_char == '.' {
                                current_state = State::InNumberAfterDot;
                            } else if !current_char.is_ascii_digit() {
                                // If it is not '0'-'9' (or a '.'), we have reached the end of the number.
                                return Ok(Some(self.construct_token_with_literal(
                                    TokenType::Number,
                                    Literal::Number(self.source[self.start..self.current_index].parse().unwrap())
                                )));
                            }
                            // If it is a digit, we stay in this state and keep consuming digits.
                        },
                        None => {
                            // If we have reached the end of the source code, then we can return with the number we constructed so far.
                            return Ok(Some(self.construct_token_with_literal(
                                TokenType::Number,
                                Literal::Number(self.source[self.start..self.current_index].parse().unwrap())
                            )));
                        }
                    }
                },
                State::InNumberAfterDot => {
                    // Similar to above, but do not allow for '.' as we already have one in the number.
                    match current_char_opt {
                        Some(current_char) => {
                            if !current_char.is_ascii_digit() {
                                // We have reached the end of the number.
                                return Ok(Some(self.construct_token_with_literal(
                                    TokenType::Number,
                                    Literal::Number(self.source[self.start..self.current_index].parse().unwrap())
                                )));
                            }
                            // If it is a digit, we stay in this state and keep consuming digits.
                        },
                        None => {
                            return Ok(Some(self.construct_token_with_literal(
                                TokenType::Number,
                                Literal::Number(self.source[self.start..self.current_index].parse().unwrap())
                            )));
                        }
                    }
                },

                State::InWord => {
                    if current_char_opt.map_or(true, |current_char| !(current_char.is_ascii_alphanumeric() || current_char == '_')) {
                        // Construct the token if:
                        // we are at the end of the source code, or
                        // if the current character is not alphanumeric or an `_`.
                        let lexeme = &self.source[self.start..self.current_index];
                        return Ok(Some(match lexeme {
                            "and" => self.construct_token(TokenType::And),
                            "break" => self.construct_token(TokenType::Break),
                            "else" => self.construct_token(TokenType::Else),
                            "false" => self.construct_token_with_literal(TokenType::False, Literal::Bool(false)),
                            "func" => self.construct_token(TokenType::Func),
                            "for" => self.construct_token(TokenType::For),
                            "if" => self.construct_token(TokenType::If),
                            "null" => self.construct_token_with_literal(TokenType::Null, Literal::Null),
                            "or" => self.construct_token(TokenType::Or),
                            "print" => self.construct_token(TokenType::Print),
                            "return" => self.construct_token(TokenType::Return),
                            "true" => self.construct_token_with_literal(TokenType::True, Literal::Bool(true)),
                            "var" => self.construct_token(TokenType::Var),
                            "while" => self.construct_token(TokenType::While),
                            _ => self.construct_token(TokenType::Identifier)
                        }));
                    }
                },
                
                State::InComment => {
                    // If we have a new line or we have reached the end of the file, the comment has ended.
                    if current_char_opt == Some('\n') {
                        self.current_line += 1;
                        current_state = State::NoOp;
                    } else if current_char_opt.is_none() {
                        current_state = State::NoOp;
                    }
                },

                State::NoOp => return Ok(None),
            }

            // Increment the pointer to the next character.
            self.current_index += 1;
        }
    }

    /// Helper function that adds a token with a literal value.
    fn construct_token_with_literal(&mut self, token_type: TokenType, literal: Literal) -> Token {
        Token {
            type_: token_type,
            lexeme: String::from(&self.source[self.start..self.current_index]),
            literal,
            line: self.current_line,
        }
    }

    /// Helper function that adds a token without a literal value.
    fn construct_token(&mut self, token_type: TokenType) -> Token {
        self.construct_token_with_literal(token_type, Literal::Null)
    }
}

#[cfg(test)]
mod tests {
    use crate::{token::{Token, TokenType, Literal}, error::ErrorType};

    use super::Tokenizer;

    fn tokenize(source: &str) -> Result<Vec<Token>, ErrorType> {
        let mut tokenizer = Tokenizer::new(source);
        tokenizer.tokenize()
    }

    #[test]
    fn one_char_tokens() {
        let source = "( ) { } [ ] : , . - % + ; / *";
        assert_eq!(Ok(vec![
            Token { type_: TokenType::LeftParen, lexeme: String::from("("), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::RightParen, lexeme: String::from(")"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::LeftCurly, lexeme: String::from("{"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::RightCurly, lexeme: String::from("}"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::LeftSquare, lexeme: String::from("["), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::RightSquare, lexeme: String::from("]"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Colon, lexeme: String::from(":"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Comma, lexeme: String::from(","), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Minus, lexeme: String::from("-"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Percent, lexeme: String::from("%"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Plus, lexeme: String::from("+"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Semicolon, lexeme: String::from(";"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Slash, lexeme: String::from("/"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Star, lexeme: String::from("*"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Eof, lexeme: String::from(""), literal: Literal::Null, line: 1 },
        ]), tokenize(source));
    }

    #[test]
    fn one_two_char_tokens() {
        let source = "! != = == > >= < <=";
        assert_eq!(Ok(vec![
            Token { type_: TokenType::Bang, lexeme: String::from("!"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::BangEqual, lexeme: String::from("!="), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Equal, lexeme: String::from("="), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::EqualEqual, lexeme: String::from("=="), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Greater, lexeme: String::from(">"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::GreaterEqual, lexeme: String::from(">="), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Less, lexeme: String::from("<"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::LessEqual, lexeme: String::from("<="), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Eof, lexeme: String::from(""), literal: Literal::Null, line: 1 },
        ]), tokenize(source));
    }

    #[test]
    fn literals() {
        let source = "\"abc\" 123 \"abc123\" 123.5 \"\" 123abc 5.5.5";
        assert_eq!(Ok(vec![
            Token { type_: TokenType::String_, lexeme: String::from("\"abc\""), literal: Literal::String_(String::from("abc")), line: 1 },
            Token { type_: TokenType::Number, lexeme: String::from("123"), literal: Literal::Number(123.0), line: 1 },
            Token { type_: TokenType::String_, lexeme: String::from("\"abc123\""), literal: Literal::String_(String::from("abc123")), line: 1 },
            Token { type_: TokenType::Number, lexeme: String::from("123.5"), literal: Literal::Number(123.5), line: 1 },
            Token { type_: TokenType::String_, lexeme: String::from("\"\""), literal: Literal::String_(String::from("")), line: 1 },
            Token { type_: TokenType::Number, lexeme: String::from("123"), literal: Literal::Number(123.0), line: 1 },
            Token { type_: TokenType::Identifier, lexeme: String::from("abc"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Number, lexeme: String::from("5.5"), literal: Literal::Number(5.5), line: 1 },
            Token { type_: TokenType::Number, lexeme: String::from("5"), literal: Literal::Number(5.0), line: 1 },
            Token { type_: TokenType::Eof, lexeme: String::from(""), literal: Literal::Null, line: 1 },
        ]), tokenize(source));
    }

    #[test]
    fn line_count() {
        let source = "12\n23";
        assert_eq!(Ok(vec![
            Token { type_: TokenType::Number, lexeme: String::from("12"), literal: Literal::Number(12.0), line: 1 },
            Token { type_: TokenType::Number, lexeme: String::from("23"), literal: Literal::Number(23.0), line: 2 },
            Token { type_: TokenType::Eof, lexeme: String::from(""), literal: Literal::Null, line: 2 },
        ]), tokenize(source));
    }

    #[test]
    fn unterminated_string() {
        let source = "\"abc\nabc\nabc";
        assert_eq!(Err(ErrorType::UnterminatedString), tokenize(source));
    }

    #[test]
    fn identifiers_and_keywords() {
        let source = "a a2 if and or ifandor";
        assert_eq!(Ok(vec![
            Token { type_: TokenType::Identifier, lexeme: String::from("a"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Identifier, lexeme: String::from("a2"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::If, lexeme: String::from("if"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::And, lexeme: String::from("and"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Or, lexeme: String::from("or"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Identifier, lexeme: String::from("ifandor"), literal: Literal::Null, line: 1 },
            Token { type_: TokenType::Eof, lexeme: String::from(""), literal: Literal::Null, line: 1 },
        ]), tokenize(source));
    }

    #[test]
    fn comments() {
        let source = "1\n#abc\n#abc\n1";
        assert_eq!(Ok(vec![
            Token { type_: TokenType::Number, lexeme: String::from("1"), literal: Literal::Number(1.0), line: 1 },
            Token { type_: TokenType::Number, lexeme: String::from("1"), literal: Literal::Number(1.0), line: 4 },
            Token { type_: TokenType::Eof, lexeme: String::from(""), literal: Literal::Null, line: 4 },
        ]), tokenize(source));
    }
}
