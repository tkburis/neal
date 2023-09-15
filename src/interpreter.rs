use crate::{environment::Environment, expr::{Expr, ExprType}, token::{Value, TokenType, Literal}, error::{ErrorType, self}, stmt::{Stmt, StmtType}};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, ast: Vec<Stmt>) {
        for stmt in &ast {
            if let Err(e) = self.execute(stmt) {
                error::report_errors(&[e]);
                return;
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), ErrorType> {
        match &stmt.stmt_type {
            StmtType::Block { body } => {
                self.environment.new_scope();
                for block_stmt in body {
                    self.execute(block_stmt)?;
                }
                self.environment.exit_scope();
                Ok(())
            },
            StmtType::Expression { expression } => {
                self.evaluate(expression)?;
                Ok(())
            },
            StmtType::Function { name, parameters, body } => {
                todo!();
                // TODO: BUILTIN FUNCTIONS like append
            },
            StmtType::If { condition, then_body, else_body } => {
                match self.evaluate(condition)? {
                    Value::Bool(condition_bool) => {
                        if condition_bool {
                            self.execute(then_body.as_ref())?;
                        } else if let Some(else_) = else_body {
                                self.execute(else_.as_ref())?;
                        }
                        Ok(())
                    },
                    _ => Err(ErrorType::IfConditionNotBoolean { line: condition.line })
                }
            },
            StmtType::Print { expression } => {
                println!("{}", self.evaluate(expression)?);
                Ok(())
            },
            StmtType::VarDecl { name, value } => {
                let value_eval = &self.evaluate(value)?;
                self.environment.declare(name.clone(), value_eval);
                Ok(())
            },
            StmtType::While { condition, body } => {
                loop {
                    let continue_ = match self.evaluate(condition)? {
                        Value::Bool(condition_bool) => {
                            condition_bool
                        },
                        _ => return Err(ErrorType::WhileConditionNotBoolean { line: stmt.line })
                    };
                    if !continue_ {
                        break;
                    }
                    self.execute(body.as_ref())?;
                }
                Ok(())
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, ErrorType> {
        match &expr.expr_type {
            ExprType::Array { elements } => {
                let values: Result<Vec<Value>, _> = elements.iter().map(|x| self.evaluate(x)).collect();
                Ok(Value::Array(values?))
            },
            ExprType::Assignment { target, value } => {
                let value_eval = self.evaluate(value.as_ref())?;
                match &target.expr_type {
                    ExprType::Element { array, index } => {
                        match &array.expr_type {
                            ExprType::Array {..} => return Err(ErrorType::InvalidAssignmentTarget { line: target.line }),
                            ExprType::Variable { name } => {
                                let index_num = self.index_expr_to_usize(index.as_ref())?;
                                self.environment.assign(name.clone(), Some(index_num), &value_eval, target.line)?;
                            },
                            _ => return Err(ErrorType::NotIndexableError { name: None, line: array.line }),
                        }
                    },
                    ExprType::Variable { name } => {
                        self.environment.assign(name.clone(), None, &value_eval, target.line)?;
                    },
                    _ => return Err(ErrorType::InvalidAssignmentTarget { line: target.line }),
                }
                Ok(value_eval)
            },
            ExprType::Binary { left, operator, right } => {
                let left_eval = self.evaluate(left.as_ref())?;
                let right_eval = self.evaluate(right.as_ref())?;

                match operator.type_ {
                    TokenType::Or |
                    TokenType::And => {
                        match (&left_eval, &right_eval) {
                            (Value::Bool(left_bool), Value::Bool(right_bool)) => {
                                match operator.type_ {
                                    TokenType::Or => Ok(Value::Bool(*left_bool || *right_bool)),
                                    TokenType::And => Ok(Value::Bool(*left_bool && *right_bool)),
                                    _ => unreachable!(),
                                }
                            },
                            (_, _) => {
                                Err(ErrorType::BinaryTypeError {
                                    expected: String::from("Boolean"),
                                    got_left: left_eval.type_to_string(),
                                    got_right: right_eval.type_to_string(),
                                    line: left.line,
                                })
                            }
                        }
                    },

                    TokenType::EqualEqual => Ok(Value::Bool(left_eval == right_eval)),
                    TokenType::BangEqual => Ok(Value::Bool(left_eval != right_eval)),

                    TokenType::Greater |
                    TokenType::Less |
                    TokenType::GreaterEqual |
                    TokenType::LessEqual => {
                        match (&left_eval, &right_eval) {
                            (Value::Number(left_num), Value::Number(right_num)) => {
                                match operator.type_ {
                                    TokenType::Greater => Ok(Value::Bool(left_num > right_num)),
                                    TokenType::Less => Ok(Value::Bool(left_num < right_num)),
                                    TokenType::GreaterEqual => Ok(Value::Bool(left_num >= right_num)),
                                    TokenType::LessEqual => Ok(Value::Bool(left_num <= right_num)),
                                    _ => unreachable!(),
                                }
                            },
                            (Value::String_(left_str), Value::String_(right_str)) => {
                                match operator.type_ {
                                    TokenType::Greater => Ok(Value::Bool(left_str > right_str)),
                                    TokenType::Less => Ok(Value::Bool(left_str < right_str)),
                                    TokenType::GreaterEqual => Ok(Value::Bool(left_str >= right_str)),
                                    TokenType::LessEqual => Ok(Value::Bool(left_str <= right_str)),
                                    _ => unreachable!(),
                                }
                            },
                            (_, _) => {
                                Err(ErrorType::BinaryTypeError {
                                    expected: String::from("Number or String"),
                                    got_left: left_eval.type_to_string(),
                                    got_right: right_eval.type_to_string(),
                                    line: left.line,
                                })
                            }
                        }
                    },

                    TokenType::Plus => {
                        match (&left_eval, &right_eval) {
                            (Value::Number(left_num), Value::Number(right_num)) => Ok(Value::Number(left_num + right_num)),
                            (Value::String_(left_str), Value::String_(right_str)) => Ok(Value::String_(format!("{}{}", left_str, right_str))),
                            (_, _) => {
                                Err(ErrorType::BinaryTypeError {
                                    expected: String::from("Number or String"),
                                    got_left: left_eval.type_to_string(),
                                    got_right: right_eval.type_to_string(),
                                    line: left.line,
                                })
                            }
                        }
                    },
                    TokenType::Minus |
                    TokenType::Star |
                    TokenType::Slash |
                    TokenType::Percent => {
                        match (&left_eval, &right_eval) {
                            (Value::Number(left_num), Value::Number(right_num)) => {
                                match operator.type_ {
                                    TokenType::Minus => Ok(Value::Number(left_num - right_num)),
                                    TokenType::Star => Ok(Value::Number(left_num * right_num)),
                                    TokenType::Slash => {
                                        if *right_num == 0.0 {
                                            Err(ErrorType::DivideByZero { line: right.line })
                                        } else {
                                            Ok(Value::Number(left_num / right_num))
                                        }
                                    },
                                    TokenType::Percent => Ok(Value::Number(left_num % right_num)),
                                    _ => unreachable!(),
                                }
                            },
                            (_, _) => {
                                Err(ErrorType::BinaryTypeError {
                                    expected: String::from("Number"),
                                    got_left: left_eval.type_to_string(),
                                    got_right: right_eval.type_to_string(),
                                    line: left.line,
                                })
                            }
                        }
                    },
                    _ => unreachable!(),
                }
            },
            ExprType::Call { callee, arguments } => {
                todo!();
            },
            ExprType::Element { array, index } => {
                let index_num = self.index_expr_to_usize(index.as_ref())?;
                match &array.expr_type {
                    ExprType::Array { elements } => {
                        if let Some(element_expr) = elements.get(index_num) {
                            self.evaluate(element_expr)
                        } else {
                            Err(ErrorType::OutOfBoundsIndexError { name: None, index: index_num, line: expr.line })
                        }
                    },
                    ExprType::Variable { name } => {
                        self.environment.get(name.clone(), Some(index_num), expr.line)
                    },
                    _ => Err(ErrorType::NotIndexableError { name: None, line: array.line }),
                }
            },
            ExprType::Grouping { expression } => {
                self.evaluate(expression.as_ref())
            },
            ExprType::Literal { value } => {
                match value {
                    Literal::Number(x) => Ok(Value::Number(*x)),
                    Literal::String_(x) => Ok(Value::String_(x.clone())),
                    Literal::Bool(x) => Ok(Value::Bool(*x)),
                    Literal::Null => Ok(Value::Null),
                }
            },
            ExprType::Unary { operator, right } => {
                let right_eval = self.evaluate(right.as_ref())?;

                match operator.type_ {
                    TokenType::Bang => {
                        match right_eval {
                            Value::Bool(right_bool) => Ok(Value::Bool(!right_bool)),
                            _ => Err(ErrorType::ExpectedTypeError {
                                expected: String::from("Boolean"),
                                got: right_eval.type_to_string(),
                                line: right.line,
                            })
                        }
                    },
                    TokenType::Minus => {
                        match right_eval {
                            Value::Number(right_num) => Ok(Value::Number(-right_num)),
                            _ => Err(ErrorType::ExpectedTypeError {
                                expected: String::from("Number"),
                                got: right_eval.type_to_string(),
                                line: right.line,
                            })
                        }
                    },
                    _ => unreachable!(),
                }
            },
            ExprType::Variable { name } => {
                self.environment.get(name.clone(), None, expr.line)
            }
        }
    }

    fn index_expr_to_usize(&mut self, index_expr: &Expr) -> Result<usize, ErrorType> {
        let index_eval = self.evaluate(index_expr)?;
        match index_eval {
            Value::Number(index_num) => {
                if index_num >= 0.0 && index_num.fract() == 0.0 {
                    Ok(index_num as usize)
                } else {
                    Err(ErrorType::NonNaturalIndexError { got: index_eval, line: index_expr.line })
                }
            },
            _ => Err(ErrorType::NonNumberIndexError { got: index_eval.type_to_string(), line: index_expr.line })
        }
    }
}
