use crate::error::{ErrorType, self};
use crate::expr::Expr;
use crate::stmt::Stmt;
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

    // TODO: possible improvement: instead of reporting error when it occurs, bubble up ErrorType and collect in a vec. Then report all at once.
    // TODO: error handling; stmts
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ErrorType> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.check_next(&[TokenType::Eof]) {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }
    
    fn statement(&mut self) -> Result<Stmt, ErrorType> {
        if self.check_and_consume(&[TokenType::For]).is_some() {
            self.for_()
        } else if self.check_and_consume(&[TokenType::Func]).is_some() {
            self.function()
        } else if self.check_and_consume(&[TokenType::If]).is_some() {
            self.if_()
        } else if self.check_and_consume(&[TokenType::Print]).is_some() {
            self.print()
        } else if self.check_and_consume(&[TokenType::Var]).is_some() {
            self.var()
        } else if self.check_and_consume(&[TokenType::While]).is_some() {
            self.while_()
        } else {
            Ok(Stmt::Expression { expression: self.expression()? })
        }
    }

    fn block(&mut self) -> Result<Stmt, ErrorType> {
        // Note this can only be called directly in e.g. `if`s and `func`s, i.e. source code cannot just have `{`s all over the place.
        self.expect(TokenType::LeftCurly, '{')?;
        
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.check_next(&[TokenType::RightCurly, TokenType::Eof]) {
            statements.push(self.statement()?);
        }

        self.expect(TokenType::RightCurly, '}')?;
        Ok(Stmt::Block { body: statements })
    }

    fn for_(&mut self) -> Result<Stmt, ErrorType> {
        self.expect(TokenType::LeftParen, '(')?;

        // The initialising statement can only either be a variable assignment or declaration.
        let mut initialiser: Option<Stmt> = None;
        if !self.check_next(&[TokenType::Semicolon]) {
            // The next token isn't a semicolon, so we've got an invalid initialising statement here.
            initialiser = Some(self.statement()?);
        }

        if self.check_and_consume(&[TokenType::Semicolon]).is_none() {
            return Err(error::report_and_return(ErrorType::ExpectedSemicolonAfterInit { line: self.current_line }));
        }

        let mut condition = Expr::Literal { value: Literal::Bool(true) };  // If there is no given condition, always `true`.
        if !self.check_next(&[TokenType::Semicolon]) {
            condition = self.expression()?;
        }

        if self.check_and_consume(&[TokenType::Semicolon]).is_none() {
            return Err(error::report_and_return(ErrorType::ExpectedSemicolonAfterCondition { line: self.current_line }));
        }

        let mut increment: Option<Stmt> = None;
        if !self.check_next(&[TokenType::RightParen]) {
            increment = Some(self.statement()?);
        }

        if self.check_and_consume(&[TokenType::RightParen]).is_none() {
            return Err(error::report_and_return(ErrorType::ExpectedParenAfterIncrement { line: self.current_line }));
        }

        let for_body = self.block()?;

        // Now we convert it to:
        //  {
        //      `initialiser`
        //      while (`condition`) {
        //          {
        //              `for_body`
        //          }
        //          `increment`
        //      }
        //  }

        let mut while_body_vec = vec![for_body];
        if let Some(inc) = increment {
            while_body_vec.push(inc);
        }

        let while_body = Stmt::Block { body: while_body_vec };

        let while_loop = Stmt::While {
            condition,
            body: Box::new(while_body)
        };

        if let Some(init) = initialiser {
            Ok(Stmt::Block { body: vec![init, while_loop] })
        } else {
            Ok(while_loop)
        }
    }

    fn function(&mut self) -> Result<Stmt, ErrorType> {
        if let Some(identifier) = self.check_and_consume(&[TokenType::Identifier]) {
            self.expect(TokenType::LeftParen, '(')?;

            let mut parameters: Vec<String> = Vec::new();
            if !self.check_next(&[TokenType::RightParen]) {
                // If there are parameters, i.e. not just ().
                loop {
                    if let Some(parameter) = self.check_and_consume(&[TokenType::Identifier]) {
                        parameters.push(parameter.lexeme);
                    } else {
                        return Err(error::report_and_return(ErrorType::ExpectedParameterName { line: self.current_line }));
                    }
                    if self.check_and_consume(&[TokenType::Comma]).is_none() {
                        break;
                    }
                }
            }

            self.expect(TokenType::RightParen, ')')?;

            let body = self.block()?;

            Ok(Stmt::Function {
                name: identifier.lexeme,
                parameters,
                body: Box::new(body),
            })
        } else {
            Err(error::report_and_return(ErrorType::ExpectedFunctionName { line: self.current_line }))
        }
    }

    fn if_(&mut self) -> Result<Stmt, ErrorType> {
        self.expect(TokenType::LeftParen, '(')?;
        let condition = self.expression()?;
        self.expect(TokenType::RightParen, ')')?;

        let then_body = self.block()?;
        
        if self.check_and_consume(&[TokenType::Else]).is_some() {
            let else_body = self.else_body()?;
            Ok(Stmt::If {
                condition,
                then_body: Box::new(then_body),
                else_body: Some(Box::new(else_body)),
            })
        } else {
            Ok(Stmt::If {
                condition,
                then_body: Box::new(then_body),
                else_body: None,
            })
        }
    }

    fn else_body(&mut self) -> Result<Stmt, ErrorType> {
        // After an `else`, there can either be another block or an `if` to make an `else if`.
        if self.check_and_consume(&[TokenType::If]).is_some() {
            // else if
            Ok(self.if_()?)
        } else {
            // else
            Ok(self.block()?)
        }
    }

    fn print(&mut self) -> Result<Stmt, ErrorType> {
        Ok(Stmt::Print { expression: self.expression()? })
    }

    fn var(&mut self) -> Result<Stmt, ErrorType> {
        if let Some(identifier) = self.check_and_consume(&[TokenType::Identifier]) {
            self.expect(TokenType::Equal, '=')?;
            
            let value = self.expression()?;
            Ok(Stmt::VarDecl {
                name: identifier.lexeme,
                value,
            })
        } else {
            Err(error::report_and_return(ErrorType::ExpectedVariableName { line: self.current_line }))
        }
    }

    fn while_(&mut self) -> Result<Stmt, ErrorType> {
        self.expect(TokenType::LeftParen, '(')?;
        let condition = self.expression()?;
        self.expect(TokenType::RightParen, ')')?;

        let body = self.block()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    // expr -> assignment
    fn expression(&mut self) -> Result<Expr, ErrorType> {
        self.assignment()
    }

    // assignment -> or "=" assignment
    // `or ("=" or)*` is NOT correct because it will build from the left.
    // E.g., `a = b = 5` -> `(a = b) = 5` which will cause problems in interpreter.
    // Given that LHS is either `Variable` or `Element`.
    fn assignment(&mut self) -> Result<Expr, ErrorType> {
        let expr = self.or()?;

        if self.check_and_consume(&[TokenType::Equal]).is_some() {
            let value = self.assignment()?;
            
            match expr {
                Expr::Element {..} |
                Expr::Variable {..} => {
                    Ok(Expr::Assignment {
                        target: Box::new(expr),
                        value: Box::new(value),
                    })
                },
                _ => {
                    Err(error::report_and_return(ErrorType::InvalidAssignmentTarget { line: self.current_line }))
                }
            }
        } else {
            Ok(expr)
        }
    }

    // or -> and ("or" and)*
    // This is equivalent to `or -> and "or" or` as order does not matter but avoids recursion in the implementation.
    // TODO: The report should use `or -> and "or" or` because this is binary.
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
            
            self.expect(TokenType::RightSquare, ']')?;
        }
        
        Ok(expr)
    }
    
    // call -> primary ("(" (expr ("," expr)*)? ")")*
    // TODO: is `a+b(c+d)` a problem?
    fn call(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.primary()?;  // This is the callee.

        while self.check_and_consume(&[TokenType::LeftParen]).is_some() {
            let mut arguments: Vec<Expr> = Vec::new();
            // TODO: infinite loop?
            if !self.check_next(&[TokenType::RightParen]) {
                // If there are arguments, i.e. not just f().
                loop {
                    arguments.push(self.expression()?);
                    if self.check_and_consume(&[TokenType::Comma]).is_none() {
                        break;
                    }
                }
            }

            self.expect(TokenType::RightParen, ')')?;

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
            self.expect(TokenType::RightParen, ')')?;
            Ok(Expr::Grouping { expression: Box::new(expr) })

        } else if self.check_and_consume(&[TokenType::LeftSquare]).is_some() {
            // Array.
            let mut elements: Vec<Expr> = Vec::new();
            
            // TODO: infinite loop?
            if !self.check_next(&[TokenType::RightSquare]) {
                // If there are elements, i.e. not just [].
                loop {
                    elements.push(self.expression()?);
                    if self.check_and_consume(&[TokenType::Comma]).is_none() {
                        break;
                    }
                }
            }

            self.expect(TokenType::RightSquare, ']')?;
            Ok(Expr::Array { elements })

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

    fn expect(&mut self, expected_type: TokenType, expected_char: char) -> Result<(), ErrorType> {
        if self.check_and_consume(&[expected_type]).is_none() {
            return Err(error::report_and_return(ErrorType::ExpectedCharacter {
                expected: expected_char,
                line: self.current_line,
            }));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{token, expr::Expr, error::ErrorType, tokenizer::Tokenizer, stmt::Stmt};

    use super::Parser;

    fn parse(source: &str) -> Result<Vec<Stmt>, ErrorType> {
        let mut tokenizer = Tokenizer::new(source);
        let tokens = tokenizer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn for_() {
        let source = "for (var x = 5; x < 10; x = x + 1) {var y = x}";
        assert_eq!(Ok(vec![Stmt::Block {
            body: vec![
                Stmt::VarDecl {
                    name: String::from("x"),
                    value: Expr::Literal { value: token::Literal::Number(5.0) },
                },
                Stmt::While {
                    condition: Expr::Binary {
                        left: Box::new(Expr::Variable { name: String::from("x") }),
                        operator: token::Token { type_: token::TokenType::Less, lexeme: String::from("<"), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr::Literal { value: token::Literal::Number(10.0) }),
                    },
                    body: Box::new(Stmt::Block {
                        body: vec![
                            Stmt::Block {
                                body: vec![
                                    Stmt::VarDecl {
                                        name: String::from("y"),
                                        value: Expr::Variable { name: String::from("x") },
                                    },
                                ],
                            },
                            Stmt::Expression { expression: Expr::Assignment {
                                target: Box::new(Expr::Variable { name: String::from("x") }),
                                value: Box::new(Expr::Binary {
                                    left: Box::new(Expr::Variable { name: String::from("x") }),
                                    operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                                    right: Box::new(Expr::Literal { value: token::Literal::Number(1.0) }),
                                }),
                            }},
                        ],
                    }),
                },
            ]
        }]), parse(source));
    }
    
    #[test]
    fn for_no_init() {
        let source = "for (; x < 10; x = x + 1) {var y = x}";
        assert_eq!(Ok(vec![
            Stmt::While {
                condition: Expr::Binary {
                    left: Box::new(Expr::Variable { name: String::from("x") }),
                    operator: token::Token { type_: token::TokenType::Less, lexeme: String::from("<"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr::Literal { value: token::Literal::Number(10.0) }),
                },
                body: Box::new(Stmt::Block {
                    body: vec![
                        Stmt::Block {
                            body: vec![
                                Stmt::VarDecl {
                                    name: String::from("y"),
                                    value: Expr::Variable { name: String::from("x") },
                                },
                            ],
                        },
                        Stmt::Expression { expression: Expr::Assignment {
                            target: Box::new(Expr::Variable { name: String::from("x") }),
                            value: Box::new(Expr::Binary {
                                left: Box::new(Expr::Variable { name: String::from("x") }),
                                operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                                right: Box::new(Expr::Literal { value: token::Literal::Number(1.0) }),
                            }),
                        }},
                    ],
                }),
            },
        ]), parse(source));
    }
    
    #[test]
    fn for_no_cond() {
        let source = "for (var x = 5;; x = x + 1) {var y = x}";
        assert_eq!(Ok(vec![Stmt::Block {
            body: vec![
                Stmt::VarDecl {
                    name: String::from("x"),
                    value: Expr::Literal { value: token::Literal::Number(5.0) },
                },
                Stmt::While {
                    condition: Expr::Literal { value: token::Literal::Bool(true) },
                    body: Box::new(Stmt::Block {
                        body: vec![
                            Stmt::Block {
                                body: vec![
                                    Stmt::VarDecl {
                                        name: String::from("y"),
                                        value: Expr::Variable { name: String::from("x") },
                                    },
                                ],
                            },
                            Stmt::Expression { expression: Expr::Assignment {
                                target: Box::new(Expr::Variable { name: String::from("x") }),
                                value: Box::new(Expr::Binary {
                                    left: Box::new(Expr::Variable { name: String::from("x") }),
                                    operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                                    right: Box::new(Expr::Literal { value: token::Literal::Number(1.0) }),
                                }),
                            }},
                        ],
                    }),
                },
            ]
        }]), parse(source));
    }
    
    #[test]
    fn for_no_inc() {
        let source = "for (var x = 5; x < 10;) {var y = x}";
        assert_eq!(Ok(vec![Stmt::Block {
            body: vec![
                Stmt::VarDecl {
                    name: String::from("x"),
                    value: Expr::Literal { value: token::Literal::Number(5.0) },
                },
                Stmt::While {
                    condition: Expr::Binary {
                        left: Box::new(Expr::Variable { name: String::from("x") }),
                        operator: token::Token { type_: token::TokenType::Less, lexeme: String::from("<"), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr::Literal { value: token::Literal::Number(10.0) }),
                    },
                    body: Box::new(Stmt::Block {
                        body: vec![
                            Stmt::Block {
                                body: vec![
                                    Stmt::VarDecl {
                                        name: String::from("y"),
                                        value: Expr::Variable { name: String::from("x") },
                                    },
                                ],
                            },
                        ],
                    }),
                },
            ]
        }]), parse(source));
    }
    
    #[test]
    fn for_no_init_semicolon() {
        let source = "for (var x = 5 x < 10; x = x + 1) {var y = x}";
        assert_eq!(Err(ErrorType::ExpectedSemicolonAfterInit { line: 1 }), parse(source));
    }
    
    #[test]
    fn for_no_cond_semicolon() {
        let source = "for (var x = 5; x < 10 x = x + 1) {var y = x}";
        assert_eq!(Err(ErrorType::ExpectedSemicolonAfterCondition { line: 1 }), parse(source));
    }
    
    #[test]
    fn unclosed_for() {
        let source = "for (var x = 5; x < 10; x = x + 1 {var y = x}";
        assert_eq!(Err(ErrorType::ExpectedParenAfterIncrement { line: 1 }), parse(source));
    }

    #[test]
    fn unopened_block() {
        let source = "for (var x = 5; x < 10; x = x + 1) var y = x}";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: '{', line: 1 }), parse(source));
    }

    #[test]
    fn unclosed_block() {
        let source = "for (var x = 5; x < 10; x = x + 1) {var y = x";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: '}', line: 1 }), parse(source));
    }
    
    #[test]
    fn func() {
        let source = "func hello(a, b) {print a print b}";
        assert_eq!(Ok(vec![Stmt::Function {
            name: String::from("hello"),
            parameters: vec![String::from("a"), String::from("b")],
            body: Box::new(Stmt::Block { body: vec![
                Stmt::Print { expression: Expr::Variable { name: String::from("a") }},
                Stmt::Print { expression: Expr::Variable { name: String::from("b") }},
            ]}),
        }]), parse(source));
    }

    #[test]
    fn func_keyword_name() {
        let source = "func print(a, b) {print a print b}";
        assert_eq!(Err(ErrorType::ExpectedFunctionName { line: 1 }), parse(source));
    }

    #[test]
    fn if_() {
        let source = "if (a == 2) {print a}";
        assert_eq!(Ok(vec![Stmt::If {
            condition: Expr::Binary {
                left: Box::new(Expr::Variable { name: String::from("a") }),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Literal { value: token::Literal::Number(2.0) }),
            },
            then_body: Box::new(Stmt::Block { body: vec![Stmt::Print { expression: Expr::Variable { name: String::from("a") } }] }),
            else_body: None,
        }]), parse(source));
    }

    #[test]
    fn else_if() {
        let source = "if (a == 2) {print a} else if (a == 3) {print b} else if (a == 4) {print c}";
        assert_eq!(Ok(vec![Stmt::If {
            condition: Expr::Binary {
                left: Box::new(Expr::Variable { name: String::from("a") }),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Literal { value: token::Literal::Number(2.0) }),
            },
            then_body: Box::new(Stmt::Block { body: vec![Stmt::Print { expression: Expr::Variable { name: String::from("a") } }] }),
            else_body: Some(Box::new(
                Stmt::If {
                    condition: Expr::Binary {
                        left: Box::new(Expr::Variable { name: String::from("a") }),
                        operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr::Literal { value: token::Literal::Number(3.0) }),
                    },
                    then_body: Box::new(Stmt::Block { body: vec![Stmt::Print { expression: Expr::Variable { name: String::from("b") } }] }),
                    else_body: Some(Box::new(
                        Stmt::If {
                            condition: Expr::Binary {
                                left: Box::new(Expr::Variable { name: String::from("a") }),
                                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                                right: Box::new(Expr::Literal { value: token::Literal::Number(4.0) }),
                            },
                            then_body: Box::new(Stmt::Block { body: vec![Stmt::Print { expression: Expr::Variable { name: String::from("c") } }] }),
                            else_body: None,
                        }
                    )),
                }
            )),
        }]), parse(source));
    }

    #[test]
    fn else_() {
        let source = "if (a == 2) {print a} else {print b}";
        assert_eq!(Ok(vec![Stmt::If {
            condition: Expr::Binary {
                left: Box::new(Expr::Variable { name: String::from("a") }),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Literal { value: token::Literal::Number(2.0) }),
            },
            then_body: Box::new(Stmt::Block { body: vec![Stmt::Print { expression: Expr::Variable { name: String::from("a") } }] }),
            else_body: Some(Box::new(Stmt::Block { body: vec![Stmt::Print { expression: Expr::Variable { name: String::from("b") } }] })),
        }]), parse(source));
    }

    #[test]
    fn print() {
        let source = "print 5*1+2*(3-4/a)";
        assert_eq!(Ok(vec![Stmt::Print { expression: Expr::Binary {
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
        }}]), parse(source));
    }

    #[test]
    fn var() {
        let source = "var a = 5";
        assert_eq!(Ok(vec![Stmt::VarDecl { name: String::from("a"), value: Expr::Literal { value: token::Literal::Number(5.0) } }]), parse(source));
    }

    #[test]
    fn invalid_var_name() {
        let source = "var 123 = 5";
        assert_eq!(Err(ErrorType::ExpectedVariableName { line: 1 }), parse(source));
    }

    #[test]
    fn while_() {
        let source = "while (a == 2) {print b}";
        assert_eq!(Ok(vec![Stmt::While {
            condition: Expr::Binary {
                left: Box::new(Expr::Variable { name: String::from("a") }),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Literal { value: token::Literal::Number(2.0) }),
            },
            body: Box::new(Stmt::Block { body: vec![Stmt::Print { expression: Expr::Variable { name: String::from("b") } }] }),
        }]), parse(source));
    }

    #[test]
    fn multiple_statements() {
        let source = "print a if (a == 2) {print a} else {print b} var c = 3";
        assert_eq!(Ok(vec![
            Stmt::Print { expression: Expr::Variable { name: String::from("a") } },
            Stmt::If {
                condition: Expr::Binary {
                    left: Box::new(Expr::Variable { name: String::from("a") }),
                    operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr::Literal { value: token::Literal::Number(2.0) }),
                },
                then_body: Box::new(Stmt::Block { body: vec![Stmt::Print { expression: Expr::Variable { name: String::from("a") } }] }),
                else_body: Some(Box::new(Stmt::Block { body: vec![Stmt::Print { expression: Expr::Variable { name: String::from("b") } }] })),
            },
            Stmt::VarDecl { name: String::from("c"), value: Expr::Literal { value: token::Literal::Number(3.0) } },
        ]), parse(source));
    }

    #[test]
    fn bidmas() {
        let source = "5*1+2*(3-4/a)";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Binary {
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
        }}]), parse(source));
    }

    #[test]
    fn logic() {
        let source = "true and true or false and true or false";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Binary {
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
        }}]), parse(source));
    }

    #[test]
    fn array() {
        let source = "[[5, a, b], 3+1, \"g\"]";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Array {
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
        }}]), parse(source));
    }
    
    #[test]
    fn empty_array() {
        let source = "[]";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Array {elements: vec![] }}]), parse(source));
    }

    #[test]
    fn unclosed_array() {
        let source = "[[5, a, b], 3+1, \"g\"";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: ']', line: 1 }), parse(source));
        let source = "[[5, a, b, 3+1, \"g\"]";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: ']', line: 1 }), parse(source));
    }
    
    #[test]
    fn error_line_numbers() {
        let source = "\n[[5, a, b, 3+1, \"g\"]";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: ']', line: 2 }), parse(source));
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
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: ')', line: 1 }), parse(source));
    }

    #[test]
    fn element() {
        let source = "a[5]";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Element {
            array: Box::new(Expr::Variable { name: String::from("a") }),
            index: 5,
        }}]), parse(source));
    }
    
    #[test]
    fn element_2d() {
        let source = "a[1][2]";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Element {
            array: Box::new(Expr::Element {
                array: Box::new(Expr::Variable { name: String::from("a") }),
                index: 1,
            }),
            index: 2,
        }}]), parse(source));
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
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Binary {
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
        }}]), parse(source));
    }

    #[test]
    fn call() {
        let source = "a(1, \"a\")(bc, 2+3)";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Call {
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
        }}]), parse(source));
    }
    
    #[test]
    fn empty_call() {
        let source = "a()";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Call {
            callee: Box::new(Expr::Variable { name: String::from("a") }),
            arguments: vec![],
        }}]), parse(source));
    }
    
    #[test]
    fn unclosed_call() {
        let source = "a(1, \"a\"(bc, 2+3)";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: ')', line: 1 }), parse(source));
        let source = "a(1, \"a\")(bc, 2+3";
        assert_eq!(Err(ErrorType::ExpectedCharacter { expected: ')', line: 1 }), parse(source));
    }

    #[test]
    fn unary() {
        let source = "!!--5";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Unary {
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
        }}]), parse(source));
    }

    #[test]
    fn etc() {
        let source = "5--4";
        assert_eq!(Ok(vec![Stmt::Expression { expression: Expr::Binary {
            left: Box::new(Expr::Literal { value: token::Literal::Number(5.0) }),
            operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr::Unary {
                operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr::Literal { value: token::Literal::Number(4.0) }),
            }),
        }}]), parse(source))
    }
}
