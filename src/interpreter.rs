use std::io::{Write, self};

use crate::environment::{Environment, Pointer, self};
use crate::expr::{Expr, ExprType};
use crate::token::{TokenType, Literal};
use crate::error::{ErrorType, self};
use crate::stmt::{Stmt, StmtType};
use crate::value::{Value, BuiltinFunction};
use crate::hash_table::HashTable;

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
                    // We cannot just use `?` here as it will exit this function call right away and not call `exit_scope()`.
                    if let Err(e) = self.execute(block_stmt) {
                        self.environment.exit_scope();
                        return Err(e);
                    }
                }
                
                // Exit and remove the scope.
                self.environment.exit_scope();
                Ok(())
            },

            StmtType::Break => {
                // Throw a `ThrownBreak` error which can be caught in the `While` statement (see below).
                // This immediately stops execution and unwinds the call stack to the nearest parent `While` statement, which emulates the behaviour of a `break` statement.
                Err(ErrorType::ThrownBreak { line: stmt.line })
            },

            StmtType::Expression { expression } => {
                // Evaluate the expression.
                // This is used for expressions with side effects, e.g., assignments and function calls.
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
                            // and the condition is `true`, execute the `then` body.
                            self.execute(then_body.as_ref())?;
                        } else if let Some(else_) = else_body {
                            // and the condition is `false`, and there is an `else` body, then execute that.
                            self.execute(else_.as_ref())?;
                        }
                        // Otherwise, do nothing.
                        Ok(())
                    },
                    // If the condition did not evaluate to a Boolean value, we cannot use it as the condition in an `If` statement.
                    // Raise a clear and specific error.
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
                        // Otherwise, it cannot be used as the condition for a loop, so raise a specific error.
                        _ => return Err(ErrorType::LoopConditionNotBoolean { line: stmt.line }),
                    };

                    // If the `condition` evaluated to `false`, stop the loop.
                    if !continue_ {
                        break;
                    }

                    match self.execute(body.as_ref()) {
                        // If the body executed with no errors, continue as normal.
                        Ok(()) => (),
                        // If a `ThrownBreak` error was thrown somewhere in the body, break the loop.
                        Err(ErrorType::ThrownBreak {..}) => break,
                        // If a different error was thrown, continue to bubble up that error.
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
                // Evaluate each expression in the array to a `Value`, and collect those in an array.
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
                    // If an error occurred (invalid assignment target), continue to bubble it up.
                    Err(e) => return Err(e),
                };

                // Evaluate to the right-hand side value, e.g., a = (b = 5) -> a = 5.
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
                                    TokenType::Or => Ok(Value::Bool(*left_bool || *right_bool)),
                                    TokenType::And => Ok(Value::Bool(*left_bool && *right_bool)),
                                    _ => unreachable!(),
                                }
                            },
                            (_, _) => {
                                // We can only perform logical operations if both sides evaluate to Booleans.
                                // If this is not the case, raise a descriptive error.
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
                        // User-defined functions.
                        if arguments.len() != parameters.len() {
                            // If the number of arguments given does not match the number of parameters expected, raise a detailed error.
                            return Err(ErrorType::ArgParamNumberMismatch {
                                arg_number: arguments.len(),
                                param_number: parameters.len(),
                                line: expr.line
                            });
                        }

                        // Iterate through the arguments and evaluate each.
                        let mut args_eval = Vec::new();
                        for arg in arguments.iter() {
                            args_eval.push(self.evaluate(arg)?);
                        }

                        // Create a new variable scope for the arguments and function execution.
                        self.environment.new_scope();

                        // Declare the arguments in the new scope.
                        for i in 0..arguments.len() {
                            self.environment.declare(parameters[i].clone(), &args_eval[i]);
                        }

                        // Execute function body.
                        let exec_result = self.execute(&body);

                        // Exit scope.
                        self.environment.exit_scope();

                        match exec_result {
                            // If the function execution did not raise any error, evaluate the call to `Null` (no return statement used in function).
                            Ok(()) => Ok(Value::Null),
                            // If the execution ended because of a raised `ThrownReturn` error, then evaluate the call to the given return vale.
                            Err(ErrorType::ThrownReturn { value, line: _ }) => Ok(value),
                            // If another error occurred, continue to bubble up the error.
                            Err(e) => Err(e),
                        }
                    },

                    Value::BuiltinFunction(function) => {
                        // Built-in functions.
                        match function {
                            BuiltinFunction::Append => {
                                // We want two arguments: the target array, and the value to append.
                                if arguments.len() != 2 {
                                    // If the number of given arguments was not 2, raise an error, providing the number of arguments received.
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 2, line: expr.line });
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
                                    // We can only append to arrays.
                                    // If `target` is not an Array variant, raise an `ExpectedTypeError` and provide the received type.
                                    Err(ErrorType::ExpectedType { expected: String::from("Array"), got: target_eval.type_to_string(), line: target.line })
                                }
                            },
                            BuiltinFunction::Input => {
                                // We want one argument: the input prompt.
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line });
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
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 2, line: expr.line });
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
                                            // Note `usize` is guaranteed to be non-negative.
                                            array.remove(index);
                                        } else {
                                            // Otherwise, raise an out-of-bounds error.
                                            return Err(ErrorType::OutOfBoundsIndex { index, line: arguments[1].line });
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
                                    _ => Err(ErrorType::ExpectedType { expected: String::from("Array or Dictionary"), got: target_eval.type_to_string(), line: target.line }),
                                }
                            },
                            BuiltinFunction::Size => {
                                // We want one argument: the target array/dictionary/string.
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line });
                                }

                                let value = self.evaluate(&arguments[0])?;
                                match value {
                                    Value::Array(array) => Ok(Value::Number(array.len() as f64)),
                                    Value::Dictionary(dict) => Ok(Value::Number(dict.size() as f64)),
                                    Value::String_(s) => Ok(Value::Number(s.len() as f64)),
                                    // If `value` did not evaluate to an Array, a Dictionary, or a String, raise an error.
                                    _ => Err(ErrorType::ExpectedType { expected: String::from("Array, Dictionary, or String"), got: value.type_to_string(), line: expr.line }),
                                }
                            },
                            BuiltinFunction::Sort => {
                                // We want one argument: the array to be sorted.
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line });
                                }

                                let value = self.evaluate(&arguments[0])?; 
                                match value {
                                    // If given argument is an array, sort using the `merge_sort` function defined below.
                                    Value::Array(array) => Ok(Value::Array(merge_sort(&array, arguments[0].line)?)),

                                    // We cannot sort objects which are not arrays, so raise an error.
                                    _ => Err(ErrorType::ExpectedType { expected: String::from("Array"), got: value.type_to_string(), line: expr.line }),
                                }
                            },
                            BuiltinFunction::ToNumber => {
                                // We want one argument: the Boolean/number/string to be converted.
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line });
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
                                            // If something went wrong during Rust's conversion, raise an error.
                                            Err(..) => Err(ErrorType::CannotConvertToNumber { line: expr.line }),
                                        }
                                    },

                                    // We can only construct numeric representations of Booleans, numbers, and strings.
                                    // If not given one of these, raise an error.
                                    _ => Err(ErrorType::ExpectedType { expected: String::from("Boolean, Number or String"), got: value.type_to_string(), line: expr.line }),
                                }
                            },
                            BuiltinFunction::ToString => {
                                // We want one argument: the Boolean/number/string to be converted.
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line });
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

                                    // We can only construct string representations of Booleans, numbers, and strings.
                                    // If not given one of these, raise an error.
                                    _ => Err(ErrorType::ExpectedType { expected: String::from("Boolean, Number or String"), got: value.type_to_string(), line: expr.line }),
                                }
                            },
                        }
                    },

                    // If the evaluated `function` was not a `Function` or a `BuiltinFunction` variant, then we cannot 'call' it.
                    // So raise an error.
                    _ => Err(ErrorType::CannotCallName { line: callee.line })
                }
            },

            ExprType::Dictionary { elements } => {
                // Create a new hash table.
                let mut hash_table = HashTable::new();

                // Iterate through the key-value pairs of the given elements.
                for key_value in elements.iter() {
                    // Evaluate each of the keys and values.
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

                match self.evaluate(array.as_ref())? {  // Evaluate `array`.
                    Value::Array(array) => {
                        // If the evaluated 'array' is an Array variant, convert the evaluated index to a `usize` index.
                        let index_num = environment::index_value_to_usize(&index_eval, index.line)?;
                        
                        // Try to get the element of `array` at index `index_num`.
                        if let Some(element) = array.get(index_num) {
                            Ok(element.clone())
                        } else {
                            // In this case, `index_num` was out of bounds.
                            Err(ErrorType::OutOfBoundsIndex { index: index_num, line: expr.line })
                        }
                    },
                    Value::Dictionary(dict) => {
                        // If the evaluated 'array' is a Dictionary variant, get value from the `HashTable` object.
                        dict.get(&index_eval, expr.line).cloned()
                    },
                    Value::String_(s) => {
                        // If the evaluated 'array' is a String variant, convert the evaluated index to a `usize` index.
                        let index_num = environment::index_value_to_usize(&index_eval, index.line)?;

                        // Try to get the character of `s` at index `index_num`.
                        if let Some(c) = s.chars().nth(index_num) {
                            Ok(Value::String_(String::from(c)))
                        } else {
                            // In this case, `index_num` was out of bounds.
                            Err(ErrorType::OutOfBoundsIndex { index: index_num, line: expr.line })
                        }
                    },
                    // If the 'array' was not an Array, a Dictionary, or a String variant, it cannot be indexed.
                    _ => Err(ErrorType::NotIndexable { line: array.line })
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
                            _ => Err(ErrorType::ExpectedType {
                                expected: String::from("Boolean"),
                                got: right_eval.type_to_string(),
                                line: right.line,
                            })
                        }
                    },
                    TokenType::Minus => {
                        // If the operator is `-`...
                        match right_eval {
                            Value::Number(right_num) => Ok(Value::Number(-right_num)),
                            // This operation only works with Number variants, so raise an `ExpectedTypeError` error otherwise.
                            // Provide the received type for clarity.
                            _ => Err(ErrorType::ExpectedType {
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
                // Recursive case.
                // E.g., a[1][2][3] -> Pointer("a", [1, 2]), [3] -> Pointer("a", [1, 2, 3])
                // So we simply add the index of the current element to the Pointer constructed in the recursion.
                let Pointer {name, indices} = self.construct_pointer(array.as_ref(), line)?;

                // Make a copy of the `indices` array and append the index of the current element.
                let mut indices_copy = indices;
                indices_copy.push(self.evaluate(index.as_ref())?);

                // Return a `Pointer` with the appended index.
                Ok(Pointer { name, indices: indices_copy })
            },
            ExprType::Variable { name } => {
                // Base case.
                // Return an empty `indices` array to be populated in the recursion.
                Ok(Pointer {name: name.clone(), indices: Vec::new()})
            },
            // Otherwise, the variant does not support assignment, so raise an error (e.g., a literal array/dictionary, a binary expression, etc.).
            _ => Err(ErrorType::InvalidAssignmentTarget { line }),
        }
    }
}

/// Sorts the given array using merge sort.
fn merge_sort(array_to_sort: &Vec<Value>, line: usize) -> Result<Vec<Value>, ErrorType> {
    let n = array_to_sort.len();

    // Base case.
    if n <= 1 {
        return Ok(array_to_sort.to_vec());
    }

    // Recursive case.

    // Recursively sort the left and right halves of the array.
    let left = merge_sort(&array_to_sort[0..n/2].to_vec(), line)?;
    let right = merge_sort(&array_to_sort[n/2..].to_vec(), line)?;

    // Merge the two sorted arrays using two pointers.
    let mut left_index = 0;
    let mut right_index = 0;
    let mut merged = Vec::new();

    while left_index < left.len() && right_index < right.len() {
        match (&left[left_index], &right[right_index]) {
            // Append the 'lower' of the two to the merged array, and advance the respective pointer.
            (Value::Number(left_num), Value::Number(right_num)) => {
                if left_num < right_num {
                    merged.push(left[left_index].clone());
                    left_index += 1;
                } else {
                    merged.push(right[right_index].clone());
                    right_index += 1;
                }
            },
            (Value::String_(left_str), Value::String_(right_str)) => {
                if left_str < right_str {
                    merged.push(left[left_index].clone());
                    left_index += 1;
                } else {
                    merged.push(right[right_index].clone());
                    right_index += 1;
                }
            },

            // We only support comparisons between numbers and between strings.
            (_, _) => {
                return Err(ErrorType::BinaryTypeError {
                    expected: String::from("Number or String"),
                    got_left: left[left_index].type_to_string(),
                    got_right: right[right_index].type_to_string(),
                    line,
                });
            }
        }
    }

    // Only one of `left` and `right` will have any elements left.
    // Append the remainder to the merged array.
    if left_index < left.len() {
        while left_index < left.len() {
            merged.push(left[left_index].clone());
            left_index += 1;
        }
    }

    if right_index < right.len() {
        while right_index < right.len() {
            merged.push(right[right_index].clone());
            right_index += 1;
        }
    }

    Ok(merged)
}
