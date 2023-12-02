use std::io::{Write, self};

use crate::{environment::{Environment, Pointer, self}, expr::{Expr, ExprType}, token::{TokenType, Literal}, error::{ErrorType, self}, stmt::{Stmt, StmtType}, value::{Value, BuiltinFunction}, hash_table::HashTable};

/// Recursively traverses the abstract syntax tree, executes statements, and evaluates expressions.
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    /// Initialises a new instance of `Interpreter`.
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    /// Executes statements in the given abstract syntax tree.
    pub fn interpret(&mut self, ast: Vec<Stmt>) {
        for stmt in &ast {
            // Iterate through each statement.
            if let Err(e) = self.execute(stmt) {
                // If an error occurred in the execution of the statement, report the error and terminate execution.
                error::report_errors(&[e]);
                return;
            }
        }
    }

    /// Executes the given statement.
    fn execute(&mut self, stmt: &Stmt) -> Result<(), ErrorType> {
        match &stmt.stmt_type {
            StmtType::Block { body } => {
                // Create a new variable scope.
                self.environment.new_scope();

                // Recursively execute each statement in the body of the `Block`.
                for block_stmt in body {
                    self.execute(block_stmt)?;
                }
                
                // Exit and remove the scope.
                self.environment.exit_scope();
                Ok(())
            },

            StmtType::Break => {
                // Throw a `ThrownBreak` error which can be caught in the `While` statement (see below).
                // This immediately stops execution and unravels the call stack to the nearest parent `While` statement,
                // which emulates the behaviour of a `break` statement.
                Err(ErrorType::ThrownBreak { line: stmt.line })
            },

            StmtType::Expression { expression } => {
                // Evaluate the expression. This is used for expressions with side effects, e.g., assignments and function calls.
                self.evaluate(expression)?;
                Ok(())
            },

            StmtType::Function { name, parameters, body } => {
                // Declare the function as a new `Value` in the environment.
                self.environment.declare(name.clone(), &Value::Function {
                    parameters: parameters.clone(),
                    body: *body.clone(),
                });
                Ok(())
            },

            StmtType::If { condition, then_body, else_body } => {
                match self.evaluate(condition)? {
                    Value::Bool(condition_bool) => {
                        // If the condition evaluated to a Boolean value...
                        if condition_bool {
                            // If the condition is `true`, execute the `then` body.
                            self.execute(then_body.as_ref())?;
                        } else if let Some(else_) = else_body {
                            // If the condition is `false`, and there is an `else` body, then execute that.
                            self.execute(else_.as_ref())?;
                        }
                        // Otherwise, do nothing.
                        Ok(())
                    },
                    // If the condition did not evaluate to a Boolean value, we cannot use it as the condition in an `If` statement.
                    // Raise a helpful and detailed error.
                    _ => Err(ErrorType::IfConditionNotBoolean { line: condition.line })
                }
            },

            StmtType::Print { expression } => {
                // Print the evaluated expression.
                println!("{}", self.evaluate(expression)?);
                Ok(())
            },

            StmtType::Return { expression } => {
                // Similar to the `Break` statement, we throw a 'dummy' error.
                // We also have to pass the value to be used as the return value of the function call.
                Err(ErrorType::ThrownReturn {
                    value: self.evaluate(expression)?,
                    line: stmt.line
                })
            },

            StmtType::VarDecl { name, value } => {
                // Evaluate the value.
                let value_eval = &self.evaluate(value)?;

                // Declare the new variable in the environment.
                self.environment.declare(name.clone(), value_eval);
                Ok(())
            },
            
            StmtType::While { condition, body } => {
                loop {
                    let continue_ = match self.evaluate(condition)? {
                        // If `condition` evaluated to a Boolean value, set `continue_` to the result of that.
                        Value::Bool(condition_bool) => condition_bool,
                        // Otherwise, it cannot be used as the condition for a loop.
                        _ => return Err(ErrorType::LoopConditionNotBoolean { line: stmt.line })
                    };

                    // If the `condition` evaluated to `false`, stop the loop.
                    if !continue_ {
                        break;
                    }

                    match self.execute(body.as_ref()) {
                        // If the body executed with no errors, continue as normal.
                        Ok(()) => (),
                        // If a `ThrownBreak` error was thrown, break the loop.
                        Err(ErrorType::ThrownBreak {..}) => break,
                        // If a different error was thrown, continue to raise that error.
                        Err(e) => return Err(e),
                    }
                }
                Ok(())
            },
        }
    }

    /// Evaluates the given expression.
    fn evaluate(&mut self, expr: &Expr) -> Result<Value, ErrorType> {
        match &expr.expr_type {
            ExprType::Array { elements } => {
                // Evaluate each expression in the array to a `Value`, and collect that in an array.
                let values: Result<Vec<Value>, _> = elements.iter().map(|x| self.evaluate(x)).collect();
                Ok(Value::Array(values?))
            },

            ExprType::Assignment { target, value } => {
                // Evaluate the value.
                let value_eval = self.evaluate(value.as_ref())?;

                // Construct the pointer to the target.
                match self.construct_pointer(target, expr.line) {
                    // Use the pointer to update the value in the environment.
                    Ok(pointer) => self.environment.update(&pointer, &value_eval, expr.line)?,
                    // If the 'base expression' is a literal, e.g., [1,2][0], then do nothing.
                    Err(ErrorType::ThrownLiteralAssignment { .. }) => (),
                    // If another error occurred, continue to raise it.
                    Err(e) => return Err(e),
                };

                // Evaluate to the right-hand side value, i.e.,
                // a = (b = 5) -> a = 5.
                Ok(value_eval)
            },

            ExprType::Binary { left, operator, right } => {
                // Evaluate the left- and right-hand side expressions.
                let left_eval = self.evaluate(left.as_ref())?;
                let right_eval = self.evaluate(right.as_ref())?;

                match operator.type_ {
                    // Perform the appropriate operation based on the type of the `operator` token.
                    TokenType::Or |
                    TokenType::And => {
                        match (&left_eval, &right_eval) {
                            (Value::Bool(left_bool), Value::Bool(right_bool)) => {
                                match operator.type_ {
                                    // ! Update report
                                    TokenType::Or =>  {
                                        if *left_bool {
                                            Ok(Value::Bool(true))
                                        } else {
                                            Ok(Value::Bool(*right_bool))
                                        }
                                    },
                                    TokenType::And => {
                                        if !*right_bool {
                                            Ok(Value::Bool(false))
                                        } else {
                                            Ok(Value::Bool(*right_bool))
                                        }
                                    },
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
                    // This is unreachable because the parser only builds Binary expressions with certain tokens.
                    _ => unreachable!(),
                }
            },

            ExprType::Call { callee, arguments } => {
                // Evaluate the callee.
                let function = self.evaluate(callee.as_ref())?;

                match function {
                    Value::Function { parameters, body } => {
                        if arguments.len() != parameters.len() {
                            // If the number of arguments given does not match the number of parameters expected,
                            // raise a detailed error.
                            return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: parameters.len(), line: expr.line })
                        }
                        
                        // Create a new variable scope.
                        self.environment.new_scope();

                        // Iterate through the arguments/parameters.
                        for i in 0..arguments.len() {
                            // Evaluate the respective argument.
                            let arg_eval = self.evaluate(&arguments[i])?;
                            // Declare the argument in the function's scope with the parameter's name.
                            self.environment.declare(parameters[i].clone(), &arg_eval);
                        }
        
                        // Execute the function body.
                        let exec_result = self.execute(&body);

                        // Exit the variable scope.
                        self.environment.exit_scope();
                        
                        match exec_result {
                            // If there was no return statement, evaluate to Null.
                            Ok(()) => Ok(Value::Null),
                            // If the execution ended in a `ThrownReturn` error, evaluate to the given value.
                            Err(ErrorType::ThrownReturn { value, line: _ }) => Ok(value),
                            // If another error occurred, continue to raise the error.
                            Err(e) => Err(e),
                        }
                    },

                    Value::BuiltinFunction(function) => {
                        match function {
                            BuiltinFunction::Append => {
                                // We want two arguments: the target array, and the value to append.
                                if arguments.len() != 2 {
                                    // If the number of given arguments was not 2, raise an error, providing the number of arguments received.
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 2, line: expr.line })
                                }

                                let target = &arguments[0];
                                let target_eval = self.evaluate(target)?;
                                let pointer = self.construct_pointer(target, target.line)?;

                                let value_eval = self.evaluate(&arguments[1])?;

                                if let Value::Array(mut array) = target_eval {
                                    // If `target` is an Array variant of Value, append and update the environment using the pointer.
                                    array.push(value_eval);
                                    self.environment.update(&pointer, &Value::Array(array.clone()), expr.line)?;

                                    // Evaluate to changed array.
                                    Ok(Value::Array(array))
                                } else {
                                    // If `target` is not an Array variant, raise an `ExpectedTypeError` and provide the received type.
                                    Err(ErrorType::ExpectedTypeError { expected: String::from("Array"), got: target_eval.type_to_string(), line: target.line })
                                }
                            },
                            BuiltinFunction::Input => {
                                // We want one argument: the input prompt.
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line })
                                }

                                // Print the input prompt.
                                let prompt = self.evaluate(&arguments[0])?;
                                print!("{}", prompt);
                                io::stdout().flush().expect("Error: flush failed");

                                // Read input.
                                let mut input = String::new();
                                io::stdin().read_line(&mut input).expect("Error: something went wrong while reading input");
                                input = input.trim().to_string();

                                // Evaluate to input string.
                                Ok(Value::String_(input))
                            },
                            BuiltinFunction::Remove => {
                                // We want two arguments: the target array/dictionary, and the index/key to remove.
                                if arguments.len() != 2 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 2, line: expr.line })
                                }

                                let target = &arguments[0];
                                let target_eval = self.evaluate(target)?;
                                let pointer = self.construct_pointer(target, target.line)?;
                                
                                let key_eval = self.evaluate(&arguments[1])?;

                                match target_eval {
                                    Value::Array(mut array) => {
                                        // If `target` is an Array variant...

                                        // Convert `key` into a `usize` index.
                                        let index = environment::index_value_to_usize(&key_eval, arguments[1].line)?;

                                        if index < array.len() {
                                            // If `index` is not out-of-bounds, perform the removal.
                                            // Note: `usize` is guaranteed to be non-negative.
                                            array.remove(index);
                                        } else {
                                            // Otherwise, raise an out-of-bounds error.
                                            return Err(ErrorType::OutOfBoundsIndexError { index, line: arguments[1].line });
                                        }
                                        
                                        // Update the environment with the new array.
                                        self.environment.update(&pointer, &Value::Array(array.clone()), expr.line)?;

                                        // Evaluate to the changed array.
                                        Ok(Value::Array(array))
                                    },
                                    Value::Dictionary(mut dict) => {
                                        // If `target` is a Dictionary variant, we can let `HashTable` take care of the removal.
                                        dict.remove(&key_eval, expr.line)?;

                                        // Update the environment with the new dictionary.
                                        self.environment.update(&pointer, &Value::Dictionary(dict.clone()), expr.line)?;
                                        
                                        // Evaluate to the changed dictionary.
                                        Ok(Value::Dictionary(dict))
                                    },
                                    // If it is not an Array or a Dictionary variant, then raise an `ExpectedTypeError`, providing the received type.
                                    _ => Err(ErrorType::ExpectedTypeError { expected: String::from("Array or Dictionary"), got: target_eval.type_to_string(), line: target.line }),
                                }
                            },
                            BuiltinFunction::Size => {
                                // We want one argument: the target array/dictionary/string.
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line })
                                }

                                let value = self.evaluate(&arguments[0])?;
                                match value {
                                    Value::Array(array) => Ok(Value::Number(array.len() as f64)),
                                    Value::Dictionary(dict) => Ok(Value::Number(dict.size() as f64)),
                                    Value::String_(s) => Ok(Value::Number(s.len() as f64)),
                                    // If `value` did not evaluate to an Array, a Dictionary, or a String, raise an error.
                                    _ => Err(ErrorType::ExpectedTypeError { expected: String::from("Array, Dictionary, or String"), got: value.type_to_string(), line: expr.line }),
                                }
                            }
                            BuiltinFunction::ToNumber => {
                                // We want one argument: the Boolean value/number/string to be converted.
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line })
                                }

                                let value = self.evaluate(&arguments[0])?;
                                match value {
                                    Value::Bool(b) => {
                                        match b {
                                            true => Ok(Value::Number(1.0)),
                                            false => Ok(Value::Number(0.0)),
                                        }
                                    },
                                    Value::Number(..) => Ok(value),
                                    Value::String_(s) => {
                                        match s.parse::<f64>() {
                                            Ok(x) => Ok(Value::Number(x)),
                                            // If something went wrong during the conversion, raise an error.
                                            Err(..) => Err(ErrorType::ConvertToNumberError { line: expr.line }),
                                        }
                                    },
                                    _ => Err(ErrorType::ExpectedTypeError { expected: String::from("Boolean, Number or String"), got: value.type_to_string(), line: expr.line }),
                                }
                            },
                            BuiltinFunction::ToString => {
                                // We want one argument: the Boolean value/number/string to be converted.
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line })
                                }

                                let value = self.evaluate(&arguments[0])?;
                                match value {
                                    Value::Bool(b) => {
                                        match b {
                                            true => Ok(Value::String_(String::from("true"))),
                                            false => Ok(Value::String_(String::from("false"))),
                                        }
                                    },
                                    Value::Number(x) => Ok(Value::String_(x.to_string())),
                                    Value::String_(..) => Ok(value),
                                    _ => Err(ErrorType::ExpectedTypeError { expected: String::from("Boolean, Number or String"), got: value.type_to_string(), line: expr.line }),
                                }
                            },
                        }
                    },
                    // If `function` was not a Function or a BuiltinFunction variant of the `Value` enum, then we cannot call it. Raise an error.
                    _ => Err(ErrorType::CannotCallName { line: callee.line })
                }
            },

            ExprType::Dictionary { elements } => {
                // Create a new hash table.
                let mut hash_table = HashTable::new();

                // Iterate through the key-value pairs of the elements.
                for key_value in elements.iter() {
                    // Evaluate each of the key and value.
                    let key_eval = self.evaluate(&key_value.key)?;
                    let value_eval = self.evaluate(&key_value.value)?;

                    // Insert the evaluated key and value into the table.
                    hash_table.insert(&key_eval, &value_eval, expr.line)?;
                }
                Ok(Value::Dictionary(hash_table))
            },

            ExprType::Element { array, index } => {
                // Note that 'array' refers to anything to the left of the index, e.g.,
                // the 'array' in `a[1][2]` is `a[1]` and the index is `2`.

                // Evaluate the index expression.
                let index_eval = self.evaluate(index.as_ref())?;

                match self.evaluate(array.as_ref())? {
                    Value::Array(array) => {
                        // If the 'array' is an Array variant, convert the evaluated index to a `usize` index.
                        let index_num = environment::index_value_to_usize(&index_eval, index.line)?;
                        
                        // Try to get the element of `array` at index `index_num`.
                        if let Some(element) = array.get(index_num) {
                            Ok(element.clone())
                        } else {
                            // In this case, `index_num` was out of bounds.
                            Err(ErrorType::OutOfBoundsIndexError { index: index_num, line: expr.line })
                        }
                    },
                    Value::Dictionary(dict) => {
                        // If the 'array' is a Dictionary variant, let `HashTable` deal with it.
                        dict.get(&index_eval, expr.line).cloned()
                    },
                    Value::String_(s) => {
                        // If the 'array' is a String variant, convert the evaluated index to a `usize` index.
                        let index_num = environment::index_value_to_usize(&index_eval, index.line)?;

                        // Try to get the character of `s` at index `index_num`.
                        if let Some(c) = s.chars().nth(index_num) {
                            Ok(Value::String_(String::from(c)))
                        } else {
                            // In this case, `index_num` was out of bounds.
                            Err(ErrorType::OutOfBoundsIndexError { index: index_num, line: expr.line })
                        }
                    },
                    // If the 'array' was not an Array, a Dictionary, or a String variant, it cannot be indexed.
                    _ => Err(ErrorType::NotIndexableError { line: array.line })
                }
            },

            ExprType::Grouping { expression } => {
                self.evaluate(expression.as_ref())
            },

            ExprType::Literal { value } => {
                // Convert a `Literal` enum into a `Value` enum.
                match value {
                    Literal::Number(x) => Ok(Value::Number(*x)),
                    Literal::String_(x) => Ok(Value::String_(x.clone())),
                    Literal::Bool(x) => Ok(Value::Bool(*x)),
                    Literal::Null => Ok(Value::Null),
                }
            },

            ExprType::Unary { operator, right } => {
                // Evaluate the right-hand side expression.
                let right_eval = self.evaluate(right.as_ref())?;

                match operator.type_ {
                    TokenType::Bang => {
                        // If the operator is `!`...
                        match right_eval {
                            Value::Bool(right_bool) => Ok(Value::Bool(!right_bool)),
                            // This operation only works with Boolean values, so raise an `ExpectedTypeError` error otherwise.
                            // Provide the received type for clarity.
                            _ => Err(ErrorType::ExpectedTypeError {
                                expected: String::from("Boolean"),
                                got: right_eval.type_to_string(),
                                line: right.line,
                            })
                        }
                    },
                    TokenType::Minus => {
                        // If the operator is `-`
                        match right_eval {
                            Value::Number(right_num) => Ok(Value::Number(-right_num)),
                            // This operation only works with Number variants, so raise an `ExpectedTypeError` error otherwise.
                            // Provide the received type for clarity.
                            _ => Err(ErrorType::ExpectedTypeError {
                                expected: String::from("Number"),
                                got: right_eval.type_to_string(),
                                line: right.line,
                            })
                        }
                    },
                    // The parser only builds `Unary` expressions with `Bang` or `Minus`, so this is unreachable.
                    _ => unreachable!(),
                }
            },

            ExprType::Variable { name } => {
                // Simply retrieve the value of the variable from the environment.
                self.environment.get(name.clone(), expr.line)
            },
        }
    }

    /// Constructs a Pointer object given an expression.
    fn construct_pointer(&mut self, element: &Expr, line: usize) -> Result<Pointer, ErrorType> {
        match &element.expr_type {
            ExprType::Element { array, index } => {
                // The recursive case.
                // E.g., a[1][2][3] -> Pointer("a", [1, 2]), [3] -> Pointer("a", [1, 2, 3])
                // So we simply add the index of the current element to the Pointer constructed in the recursion.
                let Pointer {name, indices} = self.construct_pointer(array.as_ref(), line)?;

                // Make a copy of the `indices` array and append the index of the current element.
                let mut indices_copy = indices;
                indices_copy.push(self.evaluate(index.as_ref())?);
                Ok(Pointer {name, indices: indices_copy})
            },
            ExprType::Variable { name } => {
                // The base case.
                // Return an empty `indices` array to be populated in the recursion.
                Ok(Pointer {name: name.clone(), indices: Vec::new()})
            },
            // Attempting to update literal values has no side effect as they are, by definition, stateless.
            // So we can ignore them.
            ExprType::Array {..} | ExprType::Call {..} | ExprType::Dictionary {..} => Err(ErrorType::ThrownLiteralAssignment { line }),
            // Otherwise, the variant does not support assignment, so raise an error (e.g., a binary expression).
            _ => Err(ErrorType::InvalidAssignmentTarget { line }),
        }
    }
}
