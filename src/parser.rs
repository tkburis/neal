use crate::error::{ErrorType, self};
use crate::expr::Expr;
use crate::token::{Token, TokenType, Literal};

pub struct Parser {
    tokens: Vec<Token>,
    current_index: usize,
    current_line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current_index: 0,
            current_line: 1,
        }
    }

    // TODO: error handling; stmts
    pub fn parse(&mut self) -> Result<Expr, ErrorType> {
        self.expression()
    }
    
    // pub fn program(&mut self)

    // expr -> or
    fn expression(&mut self) -> Result<Expr, ErrorType> {
        self.or()
    }

    // or -> and ("or" and)*
    // This is equivalent to `or -> and "or" or` but avoids recursion.
    fn or(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.and()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::Or]) {
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // and -> equality ("and" equality)*
    fn and(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.equality()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::And]) {
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // equality -> comparison ( ("==" | "!=") comparison)*
    fn equality(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.comparison()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // comparison -> addsub ( (">" | "<" | ">=" | "<=") addsub)*
    fn comparison(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.addsub()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::Greater, TokenType::Less, TokenType::GreaterEqual, TokenType::LessEqual]) {
            let right = self.addsub()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // addsub -> multdiv ( ("+" | "-") multdiv)*
    fn addsub(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.multdiv()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::Plus, TokenType::Minus]) {
            let right = self.multdiv()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // multdiv -> unary ( ("*" | "/") unary)*
    fn multdiv(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.unary()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::Star, TokenType::Slash]) {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // unary -> ("!" | "-") unary |
    //          element
    // This is best implemented recursively. As it is not left recursive, this is safe.
    fn unary(&mut self) -> Result<Expr, ErrorType> {
        if let Some(operator) = self.check_and_consume(&[TokenType::Bang, TokenType::Minus]) {
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.element()
        }
    }

    // element -> call ("[" integer "]")*
    fn element(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.call()?;  // This is the array.
        
        while self.check_and_consume(&[TokenType::LeftSquare]).is_some() {
            if let Some(index_token) = self.check_and_consume(&[TokenType::Number]) {
                if let Literal::Number(float) = index_token.literal {
                    if float >= 0.0 && float.fract() == 0.0 {
                        expr = Expr::Element { array: Box::new(expr), index: float as usize };
                    } else {
                        return Err(error::report_and_return(ErrorType::InvalidIndex { line: self.current_line }));
                    }
                } else {
                    return Err(error::report_and_return(ErrorType::InvalidIndex { line: self.current_line }));
                }
            } else {
                return Err(error::report_and_return(ErrorType::InvalidIndex { line: self.current_line }));
            }
            
            if self.check_and_consume(&[TokenType::RightSquare]).is_none() {
                return Err(error::report_and_return(ErrorType::ExpectedCharacter {
                    expected: String::from("]"),
                    line: self.current_line,
                }));
            }
        }
        
        Ok(expr)
    }
    
    // call -> primary ("(" (expr ("," expr)*)? ")")*
    // TODO: is `a+b(c+d)` a problem?
    fn call(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.primary()?;  // This is the callee.
        
        while self.check_and_consume(&[TokenType::LeftParen]).is_some() {
            let mut arguments: Vec<Expr> = Vec::new();
            if !self.check_next(&[TokenType::RightParen]) {
                // If there are arguments, i.e. not just f().
                loop {
                    arguments.push(self.expression()?);
                    if self.check_and_consume(&[TokenType::Comma]).is_none() {
                        break;
                    }
                }
            }

            if self.check_and_consume(&[TokenType::RightParen]).is_none() {
                return Err(error::report_and_return(ErrorType::ExpectedCharacter {
                    expected: String::from(")"),
                    line: self.current_line,
                }))
            }

            expr = Expr::Call {
                callee: Box::new(expr),
                arguments,
            }
        }

        Ok(expr)
    }

    // primary -> literals |
    //            "(" expr ")" |
	//            "[" (expr ("," expr)*)? "]" |
	//            identifier
    fn primary(&mut self) -> Result<Expr, ErrorType> {
        if self.check_and_consume(&[TokenType::True]).is_some() {
            // Literals.
            Ok(Expr::Literal { value: Literal::Bool(true) })

        } else if self.check_and_consume(&[TokenType::False]).is_some() {
            Ok(Expr::Literal { value: Literal::Bool(false) })

        } else if self.check_and_consume(&[TokenType::Null]).is_some() {
            Ok(Expr::Literal { value: Literal::Null })

        } else if let Some(token) = self.check_and_consume(&[TokenType::String_, TokenType::Number]) {
            Ok(Expr::Literal { value: token.literal })

        } else if self.check_and_consume(&[TokenType::LeftParen]).is_some() {
            // Grouping.
            let expr = self.expression()?;
            if self.check_and_consume(&[TokenType::RightParen]).is_none() {
                Err(error::report_and_return(ErrorType::ExpectedCharacter {
                    expected: String::from(")"),
                    line: self.current_line,
                }))
            } else {
                Ok(Expr::Grouping { expression: Box::new(expr) })
            }

        } else if self.check_and_consume(&[TokenType::LeftSquare]).is_some() {
            // Array.
            let mut elements: Vec<Expr> = Vec::new();
            if !self.check_next(&[TokenType::RightSquare]) {
                // If there are elements, i.e. not just [].
                loop {
                    elements.push(self.expression()?);
                    if self.check_and_consume(&[TokenType::Comma]).is_none() {
                        break;
                    }
                }
            }

            if self.check_and_consume(&[TokenType::RightSquare]).is_none() {
                Err(error::report_and_return(ErrorType::ExpectedCharacter {
                    expected: String::from("]"),
                    line: self.current_line,
                }))
            } else {
                Ok(Expr::Array { elements })
            }

        } else if let Some(identifier) = self.check_and_consume(&[TokenType::Identifier]) {
            // Variable.
            Ok(Expr::Variable { name: identifier.lexeme })

        } else {
            Err(error::report_and_return(ErrorType::ExpectedExpression { line: self.current_line }))
        }
    }

    /// Returns `Some(Token)` and advances if next token's type is one of the `expected_types`. Otherwise, or if at end of file, return `None`.
    fn check_and_consume(&mut self, expected_types: &[TokenType]) -> Option<Token> {
        if let Some(token) = self.tokens.get(self.current_index) {
            if expected_types.contains(&token.type_) {
                self.current_index += 1;
                self.current_line = token.line;
                Some(token).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns `true` if the next token's type is one of the `expected_types`. Otherwise, or if at end of file, return `false`.
    fn check_next(&self, expected_types: &[TokenType]) -> bool {
        if let Some(token) = self.tokens.get(self.current_index) {
            if expected_types.contains(&token.type_) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{token, expr::Expr, error::ErrorType, tokenizer::Tokenizer};

    use super::Parser;

    fn parse(source: &str) -> Result<Expr, ErrorType> {
        let mut tokenizer = Tokenizer::new(source);
        let tokens = tokenizer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn bidmas() {
        let source = "5*1+2*(3-4/a)";
        assert_eq!(Ok(Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal { value: token::Literal::Number(5.0) }),
                operator: token::Token { type_: token::TokenType::Star, lexeme: String::from("*"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Literal { value: token::Literal::Number(1.0) }),
            }),
            operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal { value: token::Literal::Number(2.0) }),
                operator: token::Token { type_: token::TokenType::Star, lexeme: String::from("*"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Grouping {
                    expression: Box::new(Expr::Binary {
                        left: Box::new(Expr::Literal { value: token::Literal::Number(3.0) }),
                        operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr::Binary {
                            left: Box::new(Expr::Literal { value: token::Literal::Number(4.0) }),
                            operator: token::Token { type_: token::TokenType::Slash, lexeme: String::from("/"), literal: token::Literal::Null, line: 1 },
                            right: Box::new(Expr::Variable { name: String::from("a") }),
                        }),
                    }),
                }),
            }),
        }), parse(source));
    }

    #[test]
    fn logic() {
        let source = "true and true or false and true or false";
        assert_eq!(Ok(Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {value: token::Literal::Bool(true) }),
                    operator: token::Token { type_: token::TokenType::And, lexeme: String::from("and"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr::Literal {value: token::Literal::Bool(true) }),
                }),
                operator: token::Token { type_: token::TokenType::Or, lexeme: String::from("or"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {value: token::Literal::Bool(false) }),
                    operator: token::Token { type_: token::TokenType::And, lexeme: String::from("and"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr::Literal {value: token::Literal::Bool(true) }),
                }),
            }),
            operator: token::Token { type_: token::TokenType::Or, lexeme: String::from("or"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr::Literal {value: token::Literal::Bool(false) }),
        }), parse(source));
    }

    #[test]
    fn array() {
        let source = "[[5, a, b], 3+1, \"g\"]";
        assert_eq!(Ok(Expr::Array {
            elements: vec![
                Expr::Array {
                    elements: vec![
                        Expr::Literal { value: token::Literal::Number(5.0) },
                        Expr::Variable { name: String::from("a") },
                        Expr::Variable { name: String::from("b") },
                    ]
                },
                Expr::Binary {
                    left: Box::new(Expr::Literal { value: token::Literal::Number(3.0) }),
                    operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr::Literal { value: token::Literal::Number(1.0) }),
                },
                Expr::Literal { value: token::Literal::String_(String::from("g")) },
            ]
        }), parse(source));
    }
    
    #[test]
    fn empty_array() {
        let source = "[]";
        assert_eq!(Ok(Expr::Array {elements: vec![] }), parse(source));
    }

    #[test]
    fn unclosed_array() {
        let source = "[[5, a, b], 3+1, \"g\"";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: String::from("]"), line: 1 }), parse(source));
        let source = "[[5, a, b, 3+1, \"g\"]";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: String::from("]"), line: 1 }), parse(source));
    }
    
    #[test]
    fn error_line_numbers() {
        let source = "\n[[5, a, b, 3+1, \"g\"]";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: String::from("]"), line: 2 }), parse(source));
        let source = "a[2.3]";
        assert_eq!(Err(ErrorType::InvalidIndex { line: 1 }), parse(source));
        let source = "\na[-2]";
        assert_eq!(Err(ErrorType::InvalidIndex { line: 2 }), parse(source));
        let source = "\n\na[\"abc\"]";
        assert_eq!(Err(ErrorType::InvalidIndex { line: 3 }), parse(source));
    }

    #[test]
    fn unclosed_grouping() {
        let source = "(5 + 5";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: String::from(")"), line: 1 }), parse(source));
    }

    #[test]
    fn element() {
        let source = "a[5]";
        assert_eq!(Ok(Expr::Element { array: Box::new(Expr::Variable { name: String::from("a") }), index: 5 }), parse(source));
    }
    
    #[test]
    fn element_2d() {
        let source = "a[1][2]";
        assert_eq!(Ok(Expr::Element {
            array: Box::new(Expr::Element {
                array: Box::new(Expr::Variable { name: String::from("a") }),
                index: 1,
            }),
            index: 2,
        }), parse(source));
    }
    
    #[test]
    fn invalid_index() {
        let source = "a[2.3]";
        assert_eq!(Err(ErrorType::InvalidIndex { line: 1 }), parse(source));
        let source = "a[-2]";
        assert_eq!(Err(ErrorType::InvalidIndex { line: 1 }), parse(source));
        let source = "a[\"abc\"]";
        assert_eq!(Err(ErrorType::InvalidIndex { line: 1 }), parse(source));
    }

    #[test]
    fn comparison() {
        let source = "1 < 2 == 3 > 4 <= 5 >= 6 != 7";
        assert_eq!(Ok(Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal { value: token::Literal::Number(1.0) }),
                    operator: token::Token { type_: token::TokenType::Less, lexeme: String::from("<"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr::Literal { value: token::Literal::Number(2.0) }),
                }),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Binary {
                            left: Box::new(Expr::Literal { value: token::Literal::Number(3.0) }),
                            operator: token::Token { type_: token::TokenType::Greater, lexeme: String::from(">"), literal: token::Literal::Null, line: 1 },
                            right: Box::new(Expr::Literal { value: token::Literal::Number(4.0) }),
                        }),
                        operator: token::Token { type_: token::TokenType::LessEqual, lexeme: String::from("<="), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr::Literal { value: token::Literal::Number(5.0) }),
                    }),
                    operator: token::Token { type_: token::TokenType::GreaterEqual, lexeme: String::from(">="), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr::Literal { value: token::Literal::Number(6.0) }),
                }),
            }),
            operator: token::Token { type_: token::TokenType::BangEqual, lexeme: String::from("!="), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr::Literal { value: token::Literal::Number(7.0) }),
        }), parse(source));
    }

    #[test]
    fn call() {
        let source = "a(1, \"a\")(bc, 2+3)";
        assert_eq!(Ok(Expr::Call {
            callee: Box::new(Expr::Call {
                callee: Box::new(Expr::Variable { name: String::from("a") }),
                arguments: vec![Expr::Literal { value: token::Literal::Number(1.0) }, Expr::Literal { value: token::Literal::String_(String::from("a")) }],
            }),
            arguments: vec![
                Expr::Variable { name: String::from("bc") },
                Expr::Binary {
                    left: Box::new(Expr::Literal { value: token::Literal::Number(2.0) }),
                    operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr::Literal { value: token::Literal::Number(3.0) }),
                }
            ],
        }), parse(source));
    }
    
    #[test]
    fn empty_call() {
        let source = "a()";
        assert_eq!(Ok(Expr::Call {
            callee: Box::new(Expr::Variable { name: String::from("a") }),
            arguments: vec![],
        }), parse(source));
    }
    
    #[test]
    fn unclosed_call() {
        let source = "a(1, \"a\"(bc, 2+3)";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: String::from(")"), line: 1 }), parse(source));
        let source = "a(1, \"a\")(bc, 2+3";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: String::from(")"), line: 1 }), parse(source));
    }

    #[test]
    fn unary() {
        let source = "!!--5";
        assert_eq!(Ok(Expr::Unary {
            operator: token::Token { type_: token::TokenType::Bang, lexeme: String::from("!"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr::Unary {
                operator: token::Token { type_: token::TokenType::Bang, lexeme: String::from("!"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Unary {
                    operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr::Unary {
                        operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr::Literal { value: token::Literal::Number(5.0) }),
                    }),
                }),
            }),
        }), parse(source));
    }

    #[test]
    fn etc() {
        let source = "5--4";
        assert_eq!(Ok(Expr::Binary {
            left: Box::new(Expr::Literal { value: token::Literal::Number(5.0) }),
            operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr::Unary {
                operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Literal { value: token::Literal::Number(4.0) }),
            }),
        }), parse(source))
    }
}
