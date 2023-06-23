use crate::{token::{Token, TokenType, Literal}, error::{self, ErrorType}};

pub struct Scanner<'a> {
    source: &'a str,  // Source code.
    tokens: Vec<Token>,  // Tokens that have been scanned from source code.
    start: usize,  // Points to the start of the current token.
    current: usize,  // Points to the *next* character to be scanned.
    line: usize,  // Keeps track of the current line number.
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    /// Interface function.
    /// Returns a vector of tokens if no error had taken place. Otherwise, returns `Err(())`
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, ()> {
        while !self.is_at_end() {
            // Keep scanning until we reach the end of the file.
            self.start = self.current;  // Update the start of the current token to the current character.
            self.scan_token()?;
        }

        self.tokens.push(Token {
            type_: TokenType::Eof,
            lexeme: String::from(""),
            literal: Literal::Null,
            line: self.line
        });

        Ok(self.tokens.clone())
    }

    /// Attempts to build a token from the current character(s) in the source code.
    fn scan_token(&mut self) -> Result<(), ()> {
        let c = self.advance()?;

        match c {
            // Single-character tokens.
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftCurly),
            '}' => self.add_token(TokenType::RightCurly),
            '[' => self.add_token(TokenType::LeftSquare),
            ']' => self.add_token(TokenType::RightSquare),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '/' => self.add_token(TokenType::Slash),
            '*' => self.add_token(TokenType::Star),

            // One or two character tokens. The next character has to be taken into consideration through `peek()`.
            '#' => {
                while self.peek() != Some('\n') {
                    // It is a comment. Ignore all characters until the end of the line.
                    self.advance()?;
                }
            },
            
            '!' => {
                if self.match_next('=')? {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            },
            '=' => {
                if self.match_next('=')? {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            },
            '>' => {
                if self.match_next('=')? {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            },
            '<' => {
                if self.match_next('=')? {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            },

            // Literals.
            '"' => self.string()?,
            '0'..='9' => self.number()?,

            // Identifiers or keywords.
            'a'..='z' | 'A'..='Z' | '_' => self.word()?,

            // Ignore these hidden characters.
            ' ' | '\r' | '\t' => (),

            // Increment line number.
            '\n' => self.line += 1,

            other => {
                // If the character does not match any of the above rules, raise an error.
                error::report(ErrorType::UnexpectedCharacter {
                    character: other,
                    line: self.line,
                });
                return Err(());
            },
        };

        Ok(())
    }

    /// Processes string literals.
    fn string(&mut self) -> Result<(), ()> {
        while self.peek() != Some('"') && !self.is_at_end() {
            // Keep advancing until we reach the end of the file or a `"`.
            if self.advance()? == '\n' {
                self.line += 1;
            }
        }

        if self.is_at_end() {
            // We have reached the end and there was no closing `"`.
            error::report(ErrorType::UnterminatedString);
            return Err(());
        } else {
            // Consume the closing `"`.
            self.advance()?;
            let literal = Literal::String_(self.source[self.start+1..self.current-1].to_owned());
            self.add_token_with_literal(TokenType::String_, literal);
        }

        Ok(())
    }

    /// Processes number literals.
    fn number(&mut self) -> Result<(), ()> {
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            // The above statement evaluates to `false` if `peek()` returned `None`. Otherwise, it will evaluate to the result of the closure.
            self.advance()?;
        }

        if self.peek() == Some('.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            // Consume `.` as part of the number only if it is followed by a digit.
            self.advance()?;
        }

        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            // Consume fractional part.
            self.advance()?;
        }

        let literal = Literal::Number(self.source[self.start..self.current].parse().unwrap());
        self.add_token_with_literal(TokenType::Number, literal);

        Ok(())
    }

    /// Processes identifiers and keywords.
    fn word(&mut self) -> Result<(), ()> {
        while self.peek().map_or(false, |c| c.is_ascii_alphanumeric() || c == '_') {
            // Allow alphanumeric characters and `_` in identifiers.
            self.advance()?;
        }

        let lexeme = &self.source[self.start..self.current];

        // Check if the lexeme is a keyword. If so, process as keyword. Otherwise, process as identifier.
        match lexeme {
            "and" => self.add_token(TokenType::And),
            "class" => self.add_token(TokenType::Class),
            "else" => self.add_token(TokenType::Else),
            "false" => self.add_token(TokenType::False),
            "func" => self.add_token(TokenType::Func),
            "for" => self.add_token(TokenType::For),
            "if" => self.add_token(TokenType::If),
            "null" => self.add_token(TokenType::Null),
            "or" => self.add_token(TokenType::Or),
            "print" => self.add_token(TokenType::Print),
            "return" => self.add_token(TokenType::Return),
            "self" => self.add_token(TokenType::Self_),
            "true" => self.add_token(TokenType::True),
            "var" => self.add_token(TokenType::Var),
            "while" => self.add_token(TokenType::While),
            _ => self.add_token(TokenType::Identifier)
        };

        Ok(())
    }

    /// Consumes and returns the next character pointed to by `current`.
    fn advance(&mut self) -> Result<char, ()> {
        match self.source.chars().nth(self.current) {
            Some(c) => {
                self.current += 1;
                Ok(c)
            },
            None => {
                // Somehow `current` points to something after the end. Bubble up an error.
                error::report(ErrorType::UnexpectedEof);
                Err(())
            },
        }
    }

    /// Checks if next character pointed to by `current` is `expected`. If so, consume it and return true.
    fn match_next(&mut self, expected: char) -> Result<bool, ()> {
        match self.source.chars().nth(self.current) {
            Some(c) => {
                if expected == c {
                    self.current += 1;
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            None => {
                // Somehow `current` points to something after the end. Bubble up an error.
                error::report(ErrorType::UnexpectedEof);
                Err(())
            },
        }
    }

    /// Returns the next character if there is one.
    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.current)
    }

    /// Helper function for better readability. Returns whether `current` is out of range (we have reached the end).
    fn is_at_end(&self) -> bool {
        self.peek().is_none()
    }

    /// Returns the character after next if there is one.
    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }

    /// Adds a token that does not represent a literal value.
    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, Literal::Null);
    }

    /// Adds an entire token.
    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Literal) {
        self.tokens.push(Token {
            type_: token_type,
            lexeme: String::from(&self.source[self.start..self.current]),
            literal,
            line: self.line,
        });
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]

// }
