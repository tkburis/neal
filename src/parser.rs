use crate::error::{ErrorType, self};
use crate::expr::{Expr, ExprType};
use crate::hash_table::KeyValue;
use crate::stmt::{Stmt, StmtType};
use crate::token::{Token, TokenType, Literal};

/// Performs syntax analysis.
pub struct Parser {
    tokens: Vec<Token>,
    current_index: usize,
    current_line: usize,
}

impl Parser {
    /// Initialises a new instance of `Parser`, given the sequence of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current_index: 0,
            current_line: 1,
        }
    }

    /// Returns the abstract syntax tree of the source code as a sequence of statements.
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ErrorType>> {
        let mut statements: Vec<Stmt> = Vec::new();  // The abstract syntax tree.

        // We aim to collect as many errors as possible in one run into a vector and report them all at the same time.
        let mut errors: Vec<ErrorType> = Vec::new();

        while !self.check_next(&[TokenType::Eof]) {
            // While we have not reached the end of the sequence of tokens (EOF), parse the next statement.
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(error) => {
                    // If an error occurred during the parse, collect the error and synchronise.
                    errors.push(error);
                    self.sync();
                },
            }
        }
        
        if errors.is_empty() {
            // No error occurred, return the sequence of statements.
            Ok(statements)
        } else {
            // Report all the errors and return an Err() so that the driver code knows that it cannot go on.
            error::report_errors(&errors[..]);
            Err(errors)
        }
    }

    /// Synchronises the parser to the next possible start of a new statement.
    fn sync(&mut self) {
        while !self.check_next(&[
            // These are considered tokens that are 'safe' to synchronise to.
            TokenType::Eof,
            TokenType::For,
            TokenType::Func,
            TokenType::If,
            TokenType::Print,
            TokenType::Return,
            TokenType::Var,
            TokenType::While,
        ]) {
            self.current_index += 1;  // Increment `current_index` until a 'safe' token is found.
            self.current_line = self.tokens[self.current_index].line;  // Update the line number as we iterate.
        }
    }
    
    /// Parses a statement.
    /// <statement> ::= Break | For <for> | Func <function> | If <if> | Print <print> | Return <return> | Var <var> | While <while> | <expression>
    fn statement(&mut self) -> Result<Stmt, ErrorType> {
        // If the next token is one of these, consume it and call the relevant function, which will parse the rest of the statement.
        if self.check_and_consume(&[TokenType::Break]).is_some() {
            Ok(Stmt {
                line: self.current_line,
                stmt_type: StmtType::Break
            })
        } else if self.check_and_consume(&[TokenType::For]).is_some() {
            self.for_()
        } else if self.check_and_consume(&[TokenType::Func]).is_some() {
            self.function()
        } else if self.check_and_consume(&[TokenType::If]).is_some() {
            self.if_()
        } else if self.check_and_consume(&[TokenType::Print]).is_some() {
            self.print()
        } else if self.check_and_consume(&[TokenType::Return]).is_some() {
            self.return_()
        } else if self.check_and_consume(&[TokenType::Var]).is_some() {
            self.var()
        } else if self.check_and_consume(&[TokenType::While]).is_some() {
            self.while_()
        } else {
            Ok(Stmt {
                line: self.current_line,
                stmt_type: StmtType::Expression {
                    expression: self.expression()?
                }
            })
        }
    }

    /// <block> ::= LeftCurly <statement>* RightCurly
    fn block(&mut self) -> Result<Stmt, ErrorType> {
        // Consume LeftCurly if it follows; otherwise, raise an error.
        self.expect(TokenType::LeftCurly, '{')?;
        
        // Parse <statement>*.
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.check_next(&[TokenType::RightCurly, TokenType::Eof]) {
            // Keep parsing statements until the next token is a RightCurly or we have reached the end of the sequence of tokens.
            statements.push(self.statement()?);
        }

        // Consume RightCurly.
        self.expect(TokenType::RightCurly, '}')?;
        Ok(Stmt {
            line: self.current_line,
            stmt_type: StmtType::Block {
                body: statements
            }
        })
    }

    /// <for> ::= LeftParen <statement>? Semicolon <expression>? Semicolon <statement>? RightParen <block>
    fn for_(&mut self) -> Result<Stmt, ErrorType> {
        // Consume LeftParen.
        self.expect(TokenType::LeftParen, '(')?;

        // Parse <statement>? as the initialising statement of the `for` loop. As it is optional, an Option<Stmt> is used.
        let mut initialiser: Option<Stmt> = None;
        if !self.check_next(&[TokenType::Semicolon]) {
            // If the next token is not a semicolon, we parse it as the <statement>.
            initialiser = Some(self.statement()?);
        }

        // Consume Semicolon if it follows.
        if self.check_and_consume(&[TokenType::Semicolon]).is_none() {
            // If there is no Semicolon, raise a specific error to avoid confusion as there are many semicolons in a `for` loop.
            return Err(ErrorType::ExpectedSemicolonAfterInit { line: self.current_line });
        }

        // Parse <expression>? as the condition of the `for` loop. Again, an Option<Expr> is used as it is optional.
        let mut condition = Expr {
            line: self.current_line,
            expr_type: ExprType::Literal {
                value: Literal::Bool(true)  // If no condition is given, it will be `true` by default.
            }
        };
        if !self.check_next(&[TokenType::Semicolon]) {
            // If the next token is not a Semicolon, we parse it as the <expression>.
            condition = self.expression()?;
        }

        // Consume Semicolon if it follows.
        if self.check_and_consume(&[TokenType::Semicolon]).is_none() {
            // If there is no Semicolon, again raise a specific error.
            return Err(ErrorType::ExpectedSemicolonAfterCondition { line: self.current_line });
        }

        // Parse <statement>? as the incrementing statement of the `for` loop.
        let mut increment: Option<Stmt> = None;
        if !self.check_next(&[TokenType::RightParen]) {
            increment = Some(self.statement()?);
        }

        // Consume RightParen if it is given; otherwise, raise a specific error.
        if self.check_and_consume(&[TokenType::RightParen]).is_none() {
            return Err(ErrorType::ExpectedParenAfterIncrement { line: self.current_line });
        }

        // Parse <block>, i.e., the body of the `for` loop including the curly brackets.
        let for_body = self.block()?;

        // Now, we internally convert the `for` loop into a `while` loop:
        //  {
        //      `initialiser`
        //      while (`condition`) {
        //          {
        //              `for_body`
        //          }
        //          `increment`
        //      }
        //  }

        // The body of the `while` loop includes the `for` loop body.
        let mut while_body_vec = vec![for_body];
        if let Some(inc) = increment {
            // If there is an increment, put it at the end of the `while` loop body.
            while_body_vec.push(inc);
        }

        // Create the `Block` statement for the `while` loop.
        let while_body = Stmt {
            line: self.current_line,
            stmt_type: StmtType::Block {
                body: while_body_vec
            }
        };

        // The `while` loop has the same condition as the `for` loop.
        let while_loop = Stmt {
            line: self.current_line,
            stmt_type: StmtType::While {
                condition,
                body: Box::new(while_body)
            }
        };
        
        if let Some(init) = initialiser {
            // If an initialising statement is given, place it before the `while` loop and wrap in a `Block` statement.
            Ok(Stmt {
                line: self.current_line,
                stmt_type: StmtType::Block { body: vec![init, while_loop] }
            })
        } else {
            // Otherwise, just return the `while` loop.
            Ok(while_loop)
        }
    }

    /// <function> ::= Identifier LeftParen (Identifier (Comma Identifier)*)? RightParen <block>
    fn function(&mut self) -> Result<Stmt, ErrorType> {
        if let Some(function_name_token) = self.check_and_consume(&[TokenType::Identifier]) {
            // If an Identifier was given (the name of the function), consume it.

            // Consume LeftParen.
            self.expect(TokenType::LeftParen, '(')?;

            // Parse (Identifier (Comma Identifier)*)?, i.e., collect an array of strings for the parameters.
            let mut parameters: Vec<String> = Vec::new();
            if !self.check_next(&[TokenType::RightParen]) {
                // If there are parameters, i.e., not just ().
                loop {  // Keep looping until there is no Comma following a parameter.
                    if let Some(parameter) = self.check_and_consume(&[TokenType::Identifier]) {
                        // If an Identifier was given (the name of the parameter), consume it and push it to the array of parameters.
                        parameters.push(parameter.lexeme);
                    } else {
                        // Otherwise, raise a specific error, as a parameter must be given after a comma.
                        return Err(ErrorType::ExpectedParameterName { line: self.current_line });
                    }

                    // If a Comma does not follow a parameter, then there should be no more parameters.
                    // Otherwise, if a Comma was found, consume it.
                    if self.check_and_consume(&[TokenType::Comma]).is_none() {
                        break;
                    }
                }
            }

            // Consume RightParen.
            self.expect(TokenType::RightParen, ')')?;

            // Parse <block>, the body of the function.
            let body = self.block()?;

            Ok(Stmt {
                line: self.current_line,
                stmt_type: StmtType::Function {
                    name: function_name_token.lexeme,
                    parameters,
                    body: Box::new(body),
                }
            })
        } else {
            // Otherwise, raise a specific error.
            Err(ErrorType::ExpectedFunctionName { line: self.current_line })
        }
    }

    /// <if> ::= LeftParen <expression> RightParen <block> (Else <else>)?
    fn if_(&mut self) -> Result<Stmt, ErrorType> {
        // Consume LeftParen.
        self.expect(TokenType::LeftParen, '(')?;

        // Parse <expression>, the condition of the `if` statement.
        let condition = self.expression()?;

        // Consume RightParen.
        self.expect(TokenType::RightParen, ')')?;

        // Parse <block>, the `then` body of the `if` statement.
        let then_body = self.block()?;
        
        if self.check_and_consume(&[TokenType::Else]).is_some() {
            // If there is an Else token after the `then` body, consume the Else, then parse <else>.
            let else_body = self.else_()?;
            Ok(Stmt {
                line: self.current_line,
                stmt_type: StmtType::If {
                    condition,
                    then_body: Box::new(then_body),
                    else_body: Some(Box::new(else_body)),
                }
            })
        } else {
            // Otherwise, just return the `if` statement with just the `then` body.
            Ok(Stmt {
                line: self.current_line,
                stmt_type: StmtType::If {
                    condition,
                    then_body: Box::new(then_body),
                    else_body: None,
                }
            })
        }
    }

    /// <else> ::= If <if> | <block>
    fn else_(&mut self) -> Result<Stmt, ErrorType> {
        // After an `else`, there can either be another block, which ends the `if` statement, or an `if` to make an `else if`.
        if self.check_and_consume(&[TokenType::If]).is_some() {
            // If there is an If token, consume it, then parse <if> to create an `else if`.
            Ok(self.if_()?)
        } else {
            // Otherwise, just parse the `else` block.
            Ok(self.block()?)
        }
    }

    /// <print> ::= <expression>
    fn print(&mut self) -> Result<Stmt, ErrorType> {
        Ok(Stmt {
            line: self.current_line,
            stmt_type: StmtType::Print {
                expression: self.expression()?
            }
        })
    }

    /// <return> ::= <expression>
    fn return_(&mut self) -> Result<Stmt, ErrorType> {
        Ok(Stmt {
            line: self.current_line,
            stmt_type: StmtType::Return {
                expression: self.expression()?
            }
        })
    }

    /// <var> ::= Identifier Equal <expression>
    fn var(&mut self) -> Result<Stmt, ErrorType> {
        if let Some(target_variable_token) = self.check_and_consume(&[TokenType::Identifier]) {
            // If an Identifier was given (the target variable name), consume it.

            // Consume Equal.
            self.expect(TokenType::Equal, '=')?;
            
            // Parse <expression>.
            let value = self.expression()?;
            Ok(Stmt {
                line: self.current_line,
                stmt_type: StmtType::VarDecl {
                    name: target_variable_token.lexeme,
                    value,
                }
            })
        } else {
            // Otherwise, raise a specific error.
            Err(ErrorType::ExpectedVariableName { line: self.current_line })
        }
    }

    /// <while> ::= LeftParen <expression> RightParen <block>
    fn while_(&mut self) -> Result<Stmt, ErrorType> {
        // Consume LeftParen.
        self.expect(TokenType::LeftParen, '(')?;
        
        // Parse <expression>, the condition of the `while` loop.
        let condition = self.expression()?;

        // Consume RightParen.
        self.expect(TokenType::RightParen, ')')?;

        // Parse <block>, the body of the `while` loop.
        let body = self.block()?;

        Ok(Stmt {
            line: self.current_line,
            stmt_type: StmtType::While {
                condition,
                body: Box::new(body),
            }
        })
    }

    /// Parses an expression.
    /// <expression> ::= <assignment>
    fn expression(&mut self) -> Result<Expr, ErrorType> {
        self.assignment()
    }

    /// <assignment> ::= <or> (Equal <assignment>)?
    /// Note that it is done recursively instead of <assignment> ::= <or> (Equal <or>)* as it
    /// is easier to enforce rightmost associativity with recursion on the right-hand side expression.
    /// In other words, the parse tree should look like this `a = (b = (c = 3))` as opposed to
    /// `((a = b) = c) = 3`
    fn assignment(&mut self) -> Result<Expr, ErrorType> {
        // Parse <or>, i.e., any expression with higher precedence.
        let expr = self.or()?;

        if self.check_and_consume(&[TokenType::Equal]).is_some() {
            // If an Equal was given, consume it.

            // Recursively parse <assignment>.
            let value = self.assignment()?;
            
            Ok(Expr {
                line: self.current_line,
                expr_type: ExprType::Assignment {
                    target: Box::new(expr),  // Use the <or> as the `target` of the Assignment.
                    value: Box::new(value),
                }
            })
        } else {
            // Otherwise, just return the expression as is.
            Ok(expr)
        }
    }

    /// <or> ::= <and> (Or <and>)*
    /// Note that here, iteration is used instead of recursion because we can iteratively
    /// replace `expr` with another expression, using the previous `expr` as the left-hand side.
    /// This enforces leftmost associativity and is the natural order of evaluation for binary operations,
    /// which will become relevant when two operators have the same precedence and the order of evaluation
    /// depends on the order they come in, e.g., <plus_minus>.
    /// In other words, the parse tree should look like `((a or b) or c) or d`, as opposed to
    /// `a or (b or (c or d))`. This also minimises recursion; hence, it is more memory efficient.
    fn or(&mut self) -> Result<Expr, ErrorType> {
        // Parse <and>.
        let mut expr = self.and()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::Or]) {
            // While the following token is an Or, consume it and store the token object (Or) in `operator`.

            // Parse <and>.
            let right = self.and()?;
            expr = Expr {
                line: self.current_line,
                expr_type: ExprType::Binary {
                    left: Box::new(expr),  // Use the previous `expr` as the left-hand side to enforce leftmost associativity.
                    operator,  // Store the token object (Or), as this will be used to determine the operation in runtime.
                    right: Box::new(right),
                }
            };
        }
        Ok(expr)
    }

    /// <and> ::= <equality> (And <equality>)*
    /// As above.
    fn and(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.equality()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::And]) {
            let right = self.equality()?;
            expr = Expr {
                line: self.current_line,
                expr_type: ExprType::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            };
        }
        Ok(expr)
    }

    /// <equality> ::= <comparison> ((EqualEqual | BangEqual) <comparison>)*
    /// As above.
    fn equality(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.comparison()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            // This time, allow both EqualEqual and BangEqual tokens as they have equal precedence.
            
            let right = self.comparison()?;
            expr = Expr {
                line: self.current_line,
                expr_type: ExprType::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            };
        }
        Ok(expr)
    }

    /// <comparison> ::= <plus_minus> ((Greater | Less | GreaterEqual | LessEqual) <plus_minus>)*
    /// As above.
    fn comparison(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.plus_minus()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::Greater, TokenType::Less, TokenType::GreaterEqual, TokenType::LessEqual]) {
            let right = self.plus_minus()?;
            expr = Expr {
                line: self.current_line,
                expr_type: ExprType::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            };
        }
        Ok(expr)
    }

    /// <plus_minus> ::= <star_slash_percent> ((Plus | Minus) <star_slash_percent>)*
    /// As above.
    fn plus_minus(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.star_slash_percent()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::Plus, TokenType::Minus]) {
            let right = self.star_slash_percent()?;
            expr = Expr {
                line: self.current_line,
                expr_type: ExprType::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            };
        }
        Ok(expr)
    }

    /// <star_slash_percent> ::= <unary> ((Star | Slash | Percent) <unary>)*
    /// As above.
    fn star_slash_percent(&mut self) -> Result<Expr, ErrorType> {
        let mut expr = self.unary()?;

        while let Some(operator) = self.check_and_consume(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let right = self.unary()?;
            expr = Expr {
                line: self.current_line,
                expr_type: ExprType::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            };
        }
        Ok(expr)
    }

    /// <unary> ::= (Bang | Minus) <unary> |
    ///             <element>
    fn unary(&mut self) -> Result<Expr, ErrorType> {
        if let Some(operator) = self.check_and_consume(&[TokenType::Bang, TokenType::Minus]) {
            // If the current token is either Bang or Minus, consume it.

            // Recursively parse <unary>.
            let right = self.unary()?;
            Ok(Expr {
                line: self.current_line,
                expr_type: ExprType::Unary {
                    operator,
                    right: Box::new(right),  // Use the recursion as the right-hand side expression, i.e., !(!(!(!true)))
                }
            })
        } else {
            // Otherwise, it is of lower precedence; parse <element>.
            self.element()
        }
    }

    /// <element> ::= <call> (LeftSquare <expression> RightSquare)*
    fn element(&mut self) -> Result<Expr, ErrorType> {
        // Parse <call>, i.e., the 'array' part of an element (`a` in `a[2][3]`).
        let mut expr = self.call()?;
        
        while self.check_and_consume(&[TokenType::LeftSquare]).is_some() {
            // While the following token is LeftSquare, consume it.

            // Parse <expression>, i.e., the 'index' part of an element (`1+2` in `a[1+2]`).
            let index = self.expression()?;
            expr = Expr {
                line: self.current_line,
                expr_type: ExprType::Element {
                    array: Box::new(expr),  // Use the previous `expr` as the 'array' part to keep leftmost associativity.
                    index: Box::new(index),
                }
            };

            // Consume the closing RightSquare of an index.
            self.expect(TokenType::RightSquare, ']')?;
        }
        Ok(expr)
    }
    
    /// <call> ::= <primary> (LeftParen (<expression> (Comma <expression>)*)? RightParen)*
    fn call(&mut self) -> Result<Expr, ErrorType> {
        // Parse <primary>, i.e., the callee (`f` in `f(2)(3)`).
        let mut expr = self.primary()?;

        while self.check_and_consume(&[TokenType::LeftParen]).is_some() {
            // While the following token is LeftParen, consume it.

            // Collect the arguments of the function call into an array.
            let mut arguments: Vec<Expr> = Vec::new();
            
            if !self.check_next(&[TokenType::RightParen]) {
                // If there are arguments, i.e., not just f().
                loop {
                    // Keep parsing the argument expressions and pushing them to the array of arguments...
                    arguments.push(self.expression()?);
                    if self.check_and_consume(&[TokenType::Comma]).is_none() {
                        // until the next token is not a Comma, in which case, there are no more arguments.
                        break;
                    }
                }
            }

            // Consume the closing RightParen.
            self.expect(TokenType::RightParen, ')')?;

            expr = Expr {
                line: self.current_line,
                expr_type: ExprType::Call {
                    callee: Box::new(expr),  // Use the previous `expr` as the 'callee' part to keep the leftmost associativity.
                    arguments,
                }
            }
        }
        Ok(expr)
    }


    /// <primary> ::= Literal |
    ///             LeftParen <expression> RightParen |
	///             LeftSquare (<expression> (Comma <expression>)*)? RightSquare |
    ///             LeftCurly (<expression> Colon <expression> (Comma <expression> Colon <expression>)*)? RightCurly |
	///             Identifier
    fn primary(&mut self) -> Result<Expr, ErrorType> {
        if let Some(token) = self.check_and_consume(&[
            TokenType::String_,
            TokenType::Number,
            TokenType::True,
            TokenType::False,
            TokenType::Null
        ]) {
            // Literal.
            // If the token is a String_, Number, True, False, or Null, use its literal value,
            // which is stored as an attribute in the Token object.
            Ok(Expr {
                line: self.current_line,
                expr_type: ExprType::Literal {
                    value: token.literal
                }
            })

        } else if self.check_and_consume(&[TokenType::LeftParen]).is_some() {
            // Grouping.

            // Parse <expression>.
            let expr = self.expression()?;

            // Consume the closing RightParen.
            self.expect(TokenType::RightParen, ')')?;

            Ok(Expr {
                line: self.current_line,
                expr_type: ExprType::Grouping {
                    expression: Box::new(expr)
                }
            })

        } else if self.check_and_consume(&[TokenType::LeftSquare]).is_some() {
            // Array.

            // Collect the expressions of the array elements into an array.
            let mut elements: Vec<Expr> = Vec::new();
            
            if !self.check_next(&[TokenType::RightSquare]) {
                // If there are elements, i.e., not just [].
                loop {
                    // Keep parsing the element expressions and pushing them to the array of elements...
                    elements.push(self.expression()?);
                    if self.check_and_consume(&[TokenType::Comma]).is_none() {
                        // until the next token is not a Comma, in which case,
                        // we assume there are no more elements in the array.
                        break;
                    }
                }
            }
            
            // Consume the closing RightSquare.
            self.expect(TokenType::RightSquare, ']')?;
            Ok(Expr { line: self.current_line, expr_type: ExprType::Array { elements }})

        } else if self.check_and_consume(&[TokenType::LeftCurly]).is_some() {
            // Dictionary.

            // Collect the expressions for the key-value pairs of the dictionary into an array.
            let mut elements: Vec<KeyValue<Expr>> = Vec::new();

            if !self.check_next(&[TokenType::RightCurly]) {
                // If there are elements, i.e., not just {}.
                loop {
                    // Keep parsing the key-value expressions and pushing them to the array of entries.

                    // Parse the key expression.
                    let key = self.expression()?;

                    // Consume Colon if it follows the key.
                    if self.check_and_consume(&[TokenType::Colon]).is_none() {
                        // Otherwise, raise a specific error.
                        return Err(ErrorType::ExpectedColonAfterKey { line: self.current_line });
                    }

                    // Parse the value expression.
                    let value = self.expression()?;

                    // Push them to the array of entries.
                    elements.push(KeyValue { key, value });

                    if self.check_and_consume(&[TokenType::Comma]).is_none() {
                        // Loop until the next token is not a Comma, in which case,
                        // we assume there are no more entries in the dictionary.
                        break;
                    }
                }
            }

            // Consume the closing RightCurly.
            self.expect(TokenType::RightCurly, '}')?;
            Ok(Expr { line: self.current_line, expr_type: ExprType::Dictionary { elements } })

        } else if let Some(identifier) = self.check_and_consume(&[TokenType::Identifier]) {
            // Variable.
            // If the token is an Identifier, use its stored lexeme which will be the variable name.
            // Note 'variable' in this case also means function names.
            Ok(Expr { line: self.current_line, expr_type: ExprType::Variable { name: identifier.lexeme }})

        } else {
            // If no rule matches the token, then we expected an expression but was not given one.
            // So, raise an ExpectedExpression error.
            Err(ErrorType::ExpectedExpression { line: self.current_line })
        }
    }

    /// Returns `Some(token)` and advances the pointer if the type of the next token is one of the `expected_types`.
    /// Otherwise, or if we are at the end of the sequence of tokens, return `None`.
    fn check_and_consume(&mut self, expected_types: &[TokenType]) -> Option<Token> {
        if let Some(token) = self.tokens.get(self.current_index) {
            // If we are not at the end of the sequence of tokens...
            if expected_types.contains(&token.type_) {
                // If the type of the next token is one of the `expected_types`,
                // increment `current_index` and update `current_line`.
                self.current_index += 1;
                self.current_line = token.line;
                Some(token).cloned()
            } else {
                // If it the token does not match, then return `None`.
                None
            }
        } else {
            // If we are at the end, return `None`.
            None
        }
    }

    /// Returns `true` if the type of the next token is one of the `expected_types`.
    /// Otherwise, or if we are at the end of the sequence of tokens, return `false`.
    /// The difference between this and `check_and_consume()` is that this does not advance the pointer if
    /// the token matches what is expected.
    fn check_next(&self, expected_types: &[TokenType]) -> bool {
        if let Some(token) = self.tokens.get(self.current_index) {
            // If we are not at the end of the sequence of tokens, return whether or not
            // the token's type is one of the `expected_types`.
            expected_types.contains(&token.type_)
        } else {
            // If we are at the end, return `None`.
            false
        }
    }

    /// Returns `Ok(())` and advances the pointer if the type of the next token is one of the `expected_types`.
    /// Otherwise, return `Err(ErrorType::ExpectedCharacter)`.
    /// The difference between this and `check_and_consume()` is that this does not return the token itself,
    /// just an error to be bubbled up.
    fn expect(&mut self, expected_type: TokenType, expected_char: char) -> Result<(), ErrorType> {
        if self.check_and_consume(&[expected_type]).is_none() {
            // If `check_and_consume()` returned `None`, i.e., the token does not match or we are at the end of
            // the sequence of tokens, return an `ExpectedCharacter` error.
            return Err(ErrorType::ExpectedCharacter {
                expected: expected_char,
                line: self.current_line,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{token, expr::{Expr, ExprType}, error::ErrorType, tokenizer::Tokenizer, stmt::Stmt, stmt::StmtType};

    use super::Parser;

    fn parse(source: &str) -> Result<Vec<Stmt>, Vec<ErrorType>> {
        let mut tokenizer = Tokenizer::new(source);
        let tokens = tokenizer.tokenize().expect("Tokenizer returned error.");
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    fn errors_in_result(result: Result<Vec<Stmt>, Vec<ErrorType>>, errors: Vec<ErrorType>) -> bool {
        let Err(result_errors) = result else {
            return false;
        };
        for error in errors {
            if !result_errors.contains(&error) {
                return false;
            }
        }
        true
    }

    #[test]
    fn for_() {
        let source = "for (var x = 5; x < 10; x = x + 1) {var y = x}";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Block {
            body: vec![
                Stmt { line: 1, stmt_type: StmtType::VarDecl {
                    name: String::from("x"),
                    value: Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) }},
                }},
                Stmt { line: 1, stmt_type: StmtType::While {
                    condition: Expr { line: 1, expr_type: ExprType::Binary {
                        left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }}),
                        operator: token::Token { type_: token::TokenType::Less, lexeme: String::from("<"), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(10.0) }}),
                    }},
                    body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block {
                        body: vec![
                            Stmt { line: 1, stmt_type: StmtType::Block {
                                body: vec![
                                    Stmt { line: 1, stmt_type: StmtType::VarDecl {
                                        name: String::from("y"),
                                        value: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }},
                                    }},
                                ],
                            }},
                            Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Assignment {
                                target: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }}),
                                value: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                                    left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }}),
                                    operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(1.0) }}),
                                }}),
                            }}}},
                        ],
                    }}),
                }},
            ]
        }}]), parse(source));
    }
    
    #[test]
    fn for_no_init() {
        let source = "for (; x < 10; x = x + 1) {var y = x}";
        assert_eq!(Ok(vec![
            Stmt { line: 1, stmt_type: StmtType::While {
                condition: Expr { line: 1, expr_type: ExprType::Binary {
                    left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }}),
                    operator: token::Token { type_: token::TokenType::Less, lexeme: String::from("<"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(10.0) }}),
                }},
                body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block {
                    body: vec![
                        Stmt { line: 1, stmt_type: StmtType::Block {
                            body: vec![
                                Stmt { line: 1, stmt_type: StmtType::VarDecl {
                                    name: String::from("y"),
                                    value: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }},
                                }},
                            ],
                        }},
                        Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Assignment {
                            target: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }}),
                            value: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                                left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }}),
                                operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                                right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(1.0) }}),
                            }}),
                        }}}},
                    ],
                }}),
            }},
        ]), parse(source));
    }
    
    #[test]
    fn for_no_cond() {
        let source = "for (var x = 5;; x = x + 1) {var y = x}";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Block {
            body: vec![
                Stmt { line: 1, stmt_type: StmtType::VarDecl {
                    name: String::from("x"),
                    value: Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) }},
                }},
                Stmt { line: 1, stmt_type: StmtType::While {
                    condition: Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Bool(true) }},
                    body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block {
                        body: vec![
                            Stmt { line: 1, stmt_type: StmtType::Block {
                                body: vec![
                                    Stmt { line: 1, stmt_type: StmtType::VarDecl {
                                        name: String::from("y"),
                                        value: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }},
                                    }},
                                ],
                            }},
                            Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Assignment {
                                target: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }}),
                                value: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                                    left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }}),
                                    operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(1.0) }}),
                                }}),
                            }}}},
                        ],
                    }}),
                }},
            ]
        }}]), parse(source));
    }
    
    #[test]
    fn for_no_inc() {
        let source = "for (var x = 5; x < 10;) {var y = x}";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Block {
            body: vec![
                Stmt { line: 1, stmt_type: StmtType::VarDecl {
                    name: String::from("x"),
                    value: Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) }},
                }},
                Stmt { line: 1, stmt_type: StmtType::While {
                    condition: Expr { line: 1, expr_type: ExprType::Binary {
                        left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }}),
                        operator: token::Token { type_: token::TokenType::Less, lexeme: String::from("<"), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(10.0) }}),
                    }},
                    body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block {
                        body: vec![
                            Stmt { line: 1, stmt_type: StmtType::Block {
                                body: vec![
                                    Stmt { line: 1, stmt_type: StmtType::VarDecl {
                                        name: String::from("y"),
                                        value: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("x") }},
                                    }},
                                ],
                            }},
                        ],
                    }}),
                }},
            ]
        }}]), parse(source));
    }
    
    #[test]
    fn for_no_init_semicolon() {
        let source = "for (var x = 5 x < 10; x = x + 1) {var y = x}";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedSemicolonAfterInit { line: 1 }]));
    }
    
    #[test]
    fn for_no_cond_semicolon() {
        let source = "for (var x = 5; x < 10 x = x + 1) {var y = x}";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedSemicolonAfterCondition { line: 1 }]));
    }
    
    #[test]
    fn unclosed_for() {
        let source = "for (var x = 5; x < 10; x = x + 1 {var y = x}";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedParenAfterIncrement { line: 1 }]));
    }

    #[test]
    fn unopened_block() {
        let source = "for (var x = 5; x < 10; x = x + 1) var y = x}";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedCharacter { expected: '{', line: 1 }]));
    }

    #[test]
    fn unclosed_block() {
        let source = "for (var x = 5; x < 10; x = x + 1) {var y = x";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedCharacter { expected: '}', line: 1 }]));
    }
    
    #[test]
    fn func() {
        let source = "func hello(a, b) {print a print b}";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Function {
            name: String::from("hello"),
            parameters: vec![String::from("a"), String::from("b")],
            body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![
                Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}}},
                Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("b") }}}},
            ]}}),
        }}]), parse(source));
    }

    #[test]
    fn func_keyword_name() {
        let source = "func print(a, b) {print a print b}";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedFunctionName { line: 1 }]));
    }

    #[test]
    fn if_() {
        let source = "if (a == 2) {print a}";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::If {
            condition: Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) }}),
            }},
            then_body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") } }}}] }}),
            else_body: None,
        }}]), parse(source));
    }

    #[test]
    fn else_if() {
        let source = "if (a == 2) {print a} else if (a == 3) {print b} else if (a == 4) {print c}";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::If {
            condition: Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) }}),
            }},
            then_body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") } }}}] }}),
            else_body: Some(Box::new(
                Stmt { line: 1, stmt_type: StmtType::If {
                    condition: Expr { line: 1, expr_type: ExprType::Binary {
                        left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                        operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(3.0) }}),
                    }},
                    then_body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("b") } }}}]} }),
                    else_body: Some(Box::new(
                        Stmt { line: 1, stmt_type: StmtType::If {
                            condition: Expr { line: 1, expr_type: ExprType::Binary {
                                left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                                right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(4.0) }}),
                            }},
                            then_body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("c") } }}}]} }),
                            else_body: None,
                        }}
                    )),
                }}
            )),
        }}]), parse(source));
    }

    #[test]
    fn else_() {
        let source = "if (a == 2) {print a} else {print b}";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::If {
            condition: Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) }}),
            }},
            then_body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") } }}}]} }),
            else_body: Some(Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("b") } }}}]} })),
        }}]), parse(source));
    }

    #[test]
    fn print() {
        let source = "print 5*1+2*(3-4/a)";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Binary {
            left: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) }}),
                operator: token::Token { type_: token::TokenType::Star, lexeme: String::from("*"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(1.0) }}),
            }}),
            operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) }}),
                operator: token::Token { type_: token::TokenType::Star, lexeme: String::from("*"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Grouping {
                    expression: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                        left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(3.0) }}),
                        operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                            left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(4.0) }}),
                            operator: token::Token { type_: token::TokenType::Slash, lexeme: String::from("/"), literal: token::Literal::Null, line: 1 },
                            right: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                        }}),
                    }}),
                }}),
            }}),
        }}}}]), parse(source));
    }

    #[test]
    fn var() {
        let source = "var a = 5";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::VarDecl { name: String::from("a"), value: Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) } }}}]), parse(source));
    }

    #[test]
    fn invalid_var_name() {
        let source = "var 123 = 5";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedVariableName { line: 1 }]));
    }

    #[test]
    fn while_() {
        let source = "while (a == 2) {print b}";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::While {
            condition: Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) }}),
            }},
            body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("b") } }}}]} }),
        }}]), parse(source));
    }

    #[test]
    fn multiple_statements() {
        let source = "print a if (a == 2) {print a} else {print b} var c = 3";
        assert_eq!(Ok(vec![
            Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") } } } },
            Stmt { line: 1, stmt_type: StmtType::If {
                condition: Expr { line: 1, expr_type: ExprType::Binary {
                    left: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                    operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) }}),
                }},
                then_body: Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") } }}}]} }),
                else_body: Some(Box::new(Stmt { line: 1, stmt_type: StmtType::Block { body: vec![Stmt { line: 1, stmt_type: StmtType::Print { expression: Expr { line: 1, expr_type: ExprType::Variable { name: String::from("b") } }}}]} })),
            }},
            Stmt { line: 1, stmt_type: StmtType::VarDecl { name: String::from("c"), value: Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(3.0) } } } },
        ]), parse(source));
    }

    #[test]
    fn bidmas() {
        let source = "5*1+2*(3-4/a)";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Binary {
            left: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) }}),
                operator: token::Token { type_: token::TokenType::Star, lexeme: String::from("*"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(1.0) }}),
            }}),
            operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) }}),
                operator: token::Token { type_: token::TokenType::Star, lexeme: String::from("*"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Grouping {
                    expression: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                        left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(3.0) }}),
                        operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                            left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(4.0) }}),
                            operator: token::Token { type_: token::TokenType::Slash, lexeme: String::from("/"), literal: token::Literal::Null, line: 1 },
                            right: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                    }}),
                    }}),
                }}),
            }}),
        }}}}]), parse(source));
    }

    #[test]
    fn logic() {
        let source = "true and true or false and true or false";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Binary {
            left: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                    left: Box::new(Expr { line: 1, expr_type: ExprType::Literal {value: token::Literal::Bool(true) }}),
                    operator: token::Token { type_: token::TokenType::And, lexeme: String::from("and"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal {value: token::Literal::Bool(true) }}),
                }}),
                operator: token::Token { type_: token::TokenType::Or, lexeme: String::from("or"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                    left: Box::new(Expr { line: 1, expr_type: ExprType::Literal {value: token::Literal::Bool(false) }}),
                    operator: token::Token { type_: token::TokenType::And, lexeme: String::from("and"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal {value: token::Literal::Bool(true) }}),
                }}),
            }}),
            operator: token::Token { type_: token::TokenType::Or, lexeme: String::from("or"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr { line: 1, expr_type: ExprType::Literal {value: token::Literal::Bool(false) }}),
        }}}}]), parse(source));
    }

    #[test]
    fn array() {
        let source = "[[5, a, b], 3+1, \"g\"]";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Array {
            elements: vec![
                Expr { line: 1, expr_type: ExprType::Array {
                    elements: vec![
                        Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) }},
                        Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }},
                        Expr { line: 1, expr_type: ExprType::Variable { name: String::from("b") }},
                    ]
                }},
                Expr { line: 1, expr_type: ExprType::Binary {
                    left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(3.0) }}),
                    operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(1.0) }}),
                }},
                Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::String_(String::from("g")) }},
            ]
        }}}}]), parse(source));
    }
    
    #[test]
    fn empty_array() {
        let source = "[]";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Array {elements: vec![] }}}}]), parse(source));
    }

    #[test]
    fn unclosed_array() {
        let source = "[[5, a, b], 3+1, \"g\"";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedCharacter { expected: ']', line: 1 }]));
        let source = "[[5, a, b, 3+1, \"g\"]";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedCharacter { expected: ']', line: 1 }]));
    }
    
    #[test]
    fn error_line_numbers() {
        let source = "\n[[5, a, b, 3+1, \"g\"]";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedCharacter { expected: ']', line: 2 }]));
        let source = "\n\n[[5, a, b, 3+1, \"g\"]";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedCharacter { expected: ']', line: 3 }]));
    }

    #[test]
    fn unclosed_grouping() {
        let source = "(5 + 5";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedCharacter { expected: ')', line: 1 }]));
    }

    #[test]
    fn element() {
        let source = "a[5]";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Element {
            array: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
            index: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) } }),
        }}}}]), parse(source));
    }
    
    #[test]
    fn element_2d() {
        let source = "a[1][2]";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Element {
            array: Box::new(Expr { line: 1, expr_type: ExprType::Element {
                array: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                index: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(1.0) } }),
            }}),
            index: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) } }),
        }}}}]), parse(source));
    }

    #[test]
    fn comparison() {
        let source = "1 < 2 == 3 > 4 <= 5 >= 6 != 7";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Binary {
            left: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                left: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                    left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(1.0) }}),
                    operator: token::Token { type_: token::TokenType::Less, lexeme: String::from("<"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) }}),
                }}),
                operator: token::Token { type_: token::TokenType::EqualEqual, lexeme: String::from("=="), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                    left: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                        left: Box::new(Expr { line: 1, expr_type: ExprType::Binary {
                            left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(3.0) }}),
                            operator: token::Token { type_: token::TokenType::Greater, lexeme: String::from(">"), literal: token::Literal::Null, line: 1 },
                            right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(4.0) }}),
                        }}),
                        operator: token::Token { type_: token::TokenType::LessEqual, lexeme: String::from("<="), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) }}),
                    }}),
                    operator: token::Token { type_: token::TokenType::GreaterEqual, lexeme: String::from(">="), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(6.0) }}),
                }}),
            }}),
            operator: token::Token { type_: token::TokenType::BangEqual, lexeme: String::from("!="), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(7.0) }}),
        }}}}]), parse(source));
    }

    #[test]
    fn call() {
        let source = "a(1, \"a\")(bc, 2+3)";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Call {
            callee: Box::new(Expr { line: 1, expr_type: ExprType::Call {
                callee: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
                arguments: vec![
                    Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(1.0) }},
                    Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::String_(String::from("a")) }}
                ],
            }}),
            arguments: vec![
                Expr { line: 1, expr_type: ExprType::Variable { name: String::from("bc") }},
                Expr { line: 1, expr_type: ExprType::Binary {
                    left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(2.0) }}),
                    operator: token::Token { type_: token::TokenType::Plus, lexeme: String::from("+"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(3.0) }}),
                }}
            ],
        }}}}]), parse(source));
    }
    
    #[test]
    fn empty_call() {
        let source = "a()";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Call {
            callee: Box::new(Expr { line: 1, expr_type: ExprType::Variable { name: String::from("a") }}),
            arguments: vec![],
        }}}}]), parse(source));
    }
    
    #[test]
    fn unclosed_call() {
        let source = "a(1, \"a\"(bc, 2+3)";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedCharacter { expected: ')', line: 1 }]));
        let source = "a(1, \"a\")(bc, 2+3";
        assert!(errors_in_result(parse(source), vec![ErrorType::ExpectedCharacter { expected: ')', line: 1 }]));
    }

    #[test]
    fn unary() {
        let source = "!!--5";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Unary {
            operator: token::Token { type_: token::TokenType::Bang, lexeme: String::from("!"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr { line: 1, expr_type: ExprType::Unary {
                operator: token::Token { type_: token::TokenType::Bang, lexeme: String::from("!"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Unary {
                    operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                    right: Box::new(Expr { line: 1, expr_type: ExprType::Unary {
                        operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                        right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) }}),
                    }}),
                }}),
            }}),
        }}}}]), parse(source));
    }

    #[test]
    fn etc() {
        let source = "5--4";
        assert_eq!(Ok(vec![Stmt { line: 1, stmt_type: StmtType::Expression { expression: Expr { line: 1, expr_type: ExprType::Binary {
            left: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(5.0) }}),
            operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
            right: Box::new(Expr { line: 1, expr_type: ExprType::Unary {
                operator: token::Token { type_: token::TokenType::Minus, lexeme: String::from("-"), literal: token::Literal::Null, line: 1 },
                right: Box::new(Expr { line: 1, expr_type: ExprType::Literal { value: token::Literal::Number(4.0) }}),
            }}),
        }}}}]), parse(source));
    }

    #[test]
    fn sync() {
        let source = "print {\nfor (x = 5; x < 2; x = x + 1 {print x}";
        assert!(errors_in_result(parse(source), vec![
            ErrorType::ExpectedExpression { line: 1 },
            ErrorType::ExpectedParenAfterIncrement { line: 2 },
        ]));
    }
}
