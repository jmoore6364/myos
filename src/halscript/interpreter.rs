use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use super::parser::{Stmt, Expr, BinOp, UnOp};
use super::value::Value;
use crate::println;

#[derive(Debug, Clone)]
pub struct Function {
    params: Vec<String>,
    body: Vec<Stmt>,
}

pub struct Interpreter {
    globals: BTreeMap<String, Value>,
    functions: BTreeMap<String, Function>,
    locals: Vec<BTreeMap<String, Value>>,
    return_value: Option<Value>,
    break_flag: bool,
    continue_flag: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals: BTreeMap::new(),
            functions: BTreeMap::new(),
            locals: Vec::new(),
            return_value: None,
            break_flag: false,
            continue_flag: false,
        }
    }

    pub fn run(&mut self, statements: Vec<Stmt>) -> Result<(), String> {
        for stmt in statements {
            self.execute_stmt(stmt)?;
            if self.return_value.is_some() {
                break;
            }
        }
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let(name, expr) => {
                let value = self.eval_expr(expr)?;
                self.set_var(name, value);
                Ok(())
            }
            Stmt::Assign(name, expr) => {
                let value = self.eval_expr(expr)?;
                self.set_var(name, value);
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.eval_expr(expr)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let cond_value = self.eval_expr(condition)?;
                if cond_value.is_truthy() {
                    for stmt in then_branch {
                        self.execute_stmt(stmt)?;
                        if self.return_value.is_some() {
                            break;
                        }
                    }
                } else if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.execute_stmt(stmt)?;
                        if self.return_value.is_some() {
                            break;
                        }
                    }
                }
                Ok(())
            }
            Stmt::While(condition, body) => {
                while self.eval_expr(condition.clone())?.is_truthy() {
                    for stmt in &body {
                        self.execute_stmt(stmt.clone())?;
                        if self.return_value.is_some() {
                            return Ok(());
                        }
                        if self.break_flag {
                            self.break_flag = false;
                            return Ok(());
                        }
                        if self.continue_flag {
                            self.continue_flag = false;
                            break;
                        }
                    }
                    if self.break_flag {
                        self.break_flag = false;
                        break;
                    }
                }
                Ok(())
            }
            Stmt::For(var, start, end, body) => {
                let start_val = self.eval_expr(start)?
                    .to_number()
                    .ok_or("For loop start must be a number")?;
                let end_val = self.eval_expr(end)?
                    .to_number()
                    .ok_or("For loop end must be a number")?;

                for i in start_val..end_val {
                    self.set_var(var.clone(), Value::Number(i));
                    for stmt in &body {
                        self.execute_stmt(stmt.clone())?;
                        if self.return_value.is_some() {
                            return Ok(());
                        }
                        if self.break_flag {
                            self.break_flag = false;
                            return Ok(());
                        }
                        if self.continue_flag {
                            self.continue_flag = false;
                            break;
                        }
                    }
                    if self.break_flag {
                        self.break_flag = false;
                        break;
                    }
                }
                Ok(())
            }
            Stmt::Return(expr) => {
                self.return_value = Some(if let Some(e) = expr {
                    self.eval_expr(e)?
                } else {
                    Value::Null
                });
                Ok(())
            }
            Stmt::Break => {
                self.break_flag = true;
                Ok(())
            }
            Stmt::Continue => {
                self.continue_flag = true;
                Ok(())
            }
            Stmt::Function(name, params, body) => {
                self.functions.insert(name, Function { params, body });
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(())
            }
        }
    }

    fn eval_expr(&mut self, expr: Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal(value) => Ok(value),
            Expr::Ident(name) => self.get_var(&name),
            Expr::Binary(left, op, right) => {
                let left_val = self.eval_expr(*left)?;
                let right_val = self.eval_expr(*right)?;
                self.eval_binary(left_val, op, right_val)
            }
            Expr::Unary(op, expr) => {
                let value = self.eval_expr(*expr)?;
                match op {
                    UnOp::Neg => {
                        if let Some(n) = value.to_number() {
                            Ok(Value::Number(-n))
                        } else {
                            Err(format!("Cannot negate {:?}", value))
                        }
                    }
                    UnOp::Not => Ok(Value::Bool(!value.is_truthy())),
                }
            }
            Expr::Call(name, args) => self.call_function(name, args),
            Expr::Index(arr_expr, index_expr) => {
                let arr = self.eval_expr(*arr_expr)?;
                let index = self.eval_expr(*index_expr)?;

                if let Value::Array(elements) = arr {
                    if let Some(idx) = index.to_number() {
                        if idx >= 0 && (idx as usize) < elements.len() {
                            Ok(elements[idx as usize].clone())
                        } else {
                            Err(format!("Index {} out of bounds", idx))
                        }
                    } else {
                        Err(String::from("Array index must be a number"))
                    }
                } else {
                    Err(String::from("Can only index arrays"))
                }
            }
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_expr(elem)?);
                }
                Ok(Value::Array(values))
            }
        }
    }

    fn eval_binary(&self, left: Value, op: BinOp, right: Value) -> Result<Value, String> {
        use core::cmp::Ordering;

        match op {
            BinOp::Add => left.add(&right),
            BinOp::Sub => left.sub(&right),
            BinOp::Mul => left.mul(&right),
            BinOp::Div => left.div(&right),
            BinOp::Mod => {
                match (left.to_number(), right.to_number()) {
                    (Some(a), Some(b)) if b != 0 => Ok(Value::Number(a % b)),
                    (Some(_), Some(0)) => Err(String::from("Modulo by zero")),
                    _ => Err(format!("Cannot modulo {:?} by {:?}", left, right)),
                }
            }
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::NotEq => Ok(Value::Bool(left != right)),
            BinOp::Lt => {
                match left.compare(&right) {
                    Some(Ordering::Less) => Ok(Value::Bool(true)),
                    Some(_) => Ok(Value::Bool(false)),
                    None => Err(format!("Cannot compare {:?} and {:?}", left, right)),
                }
            }
            BinOp::Gt => {
                match left.compare(&right) {
                    Some(Ordering::Greater) => Ok(Value::Bool(true)),
                    Some(_) => Ok(Value::Bool(false)),
                    None => Err(format!("Cannot compare {:?} and {:?}", left, right)),
                }
            }
            BinOp::LtEq => {
                match left.compare(&right) {
                    Some(Ordering::Less) | Some(Ordering::Equal) => Ok(Value::Bool(true)),
                    Some(_) => Ok(Value::Bool(false)),
                    None => Err(format!("Cannot compare {:?} and {:?}", left, right)),
                }
            }
            BinOp::GtEq => {
                match left.compare(&right) {
                    Some(Ordering::Greater) | Some(Ordering::Equal) => Ok(Value::Bool(true)),
                    Some(_) => Ok(Value::Bool(false)),
                    None => Err(format!("Cannot compare {:?} and {:?}", left, right)),
                }
            }
            BinOp::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            BinOp::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),
        }
    }

    fn call_function(&mut self, name: String, args: Vec<Expr>) -> Result<Value, String> {
        // Built-in functions
        match name.as_str() {
            "len" => {
                if args.len() != 1 {
                    return Err(format!("len() takes 1 argument, got {}", args.len()));
                }
                let value = self.eval_expr(args[0].clone())?;
                match value {
                    Value::String(s) => Ok(Value::Number(s.len() as i64)),
                    Value::Array(a) => Ok(Value::Number(a.len() as i64)),
                    _ => Err(String::from("len() requires string or array")),
                }
            }
            "uptime" => {
                Ok(Value::Number(crate::time::uptime_seconds() as i64))
            }
            "abs" => {
                if args.len() != 1 {
                    return Err(format!("abs() takes 1 argument, got {}", args.len()));
                }
                let value = self.eval_expr(args[0].clone())?;
                match value.to_number() {
                    Some(n) => Ok(Value::Number(n.abs())),
                    None => Err(String::from("abs() requires a number")),
                }
            }
            "min" => {
                if args.len() != 2 {
                    return Err(format!("min() takes 2 arguments, got {}", args.len()));
                }
                let a = self.eval_expr(args[0].clone())?.to_number()
                    .ok_or("min() requires numbers")?;
                let b = self.eval_expr(args[1].clone())?.to_number()
                    .ok_or("min() requires numbers")?;
                Ok(Value::Number(if a < b { a } else { b }))
            }
            "max" => {
                if args.len() != 2 {
                    return Err(format!("max() takes 2 arguments, got {}", args.len()));
                }
                let a = self.eval_expr(args[0].clone())?.to_number()
                    .ok_or("max() requires numbers")?;
                let b = self.eval_expr(args[1].clone())?.to_number()
                    .ok_or("max() requires numbers")?;
                Ok(Value::Number(if a > b { a } else { b }))
            }
            "pow" => {
                if args.len() != 2 {
                    return Err(format!("pow() takes 2 arguments, got {}", args.len()));
                }
                let base = self.eval_expr(args[0].clone())?.to_number()
                    .ok_or("pow() requires numbers")?;
                let exp = self.eval_expr(args[1].clone())?.to_number()
                    .ok_or("pow() requires numbers")?;

                if exp < 0 {
                    return Err(String::from("pow() does not support negative exponents"));
                }

                let mut result = 1i64;
                for _ in 0..exp {
                    result *= base;
                }
                Ok(Value::Number(result))
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Err(format!("sqrt() takes 1 argument, got {}", args.len()));
                }
                let n = self.eval_expr(args[0].clone())?.to_number()
                    .ok_or("sqrt() requires a number")?;

                if n < 0 {
                    return Err(String::from("sqrt() of negative number"));
                }

                // Integer square root using Newton's method
                if n == 0 { return Ok(Value::Number(0)); }
                let mut x = n;
                let mut y = (x + 1) / 2;
                while y < x {
                    x = y;
                    y = (x + n / x) / 2;
                }
                Ok(Value::Number(x))
            }
            "str" => {
                if args.len() != 1 {
                    return Err(format!("str() takes 1 argument, got {}", args.len()));
                }
                let value = self.eval_expr(args[0].clone())?;
                Ok(Value::String(value.to_string()))
            }
            "num" => {
                if args.len() != 1 {
                    return Err(format!("num() takes 1 argument, got {}", args.len()));
                }
                let value = self.eval_expr(args[0].clone())?;
                match value {
                    Value::Number(n) => Ok(Value::Number(n)),
                    Value::String(s) => {
                        s.parse::<i64>()
                            .map(Value::Number)
                            .map_err(|_| format!("Cannot convert '{}' to number", s))
                    }
                    Value::Bool(b) => Ok(Value::Number(if b { 1 } else { 0 })),
                    _ => Err(String::from("Cannot convert to number")),
                }
            }
            "range" => {
                if args.len() != 2 {
                    return Err(format!("range() takes 2 arguments, got {}", args.len()));
                }
                let start = self.eval_expr(args[0].clone())?.to_number()
                    .ok_or("range() requires numbers")?;
                let end = self.eval_expr(args[1].clone())?.to_number()
                    .ok_or("range() requires numbers")?;

                let mut arr = Vec::new();
                for i in start..end {
                    arr.push(Value::Number(i));
                }
                Ok(Value::Array(arr))
            }
            "push" => {
                if args.len() != 2 {
                    return Err(format!("push() takes 2 arguments, got {}", args.len()));
                }
                let arr_expr = &args[0];
                let value = self.eval_expr(args[1].clone())?;

                // This is a limitation - we can't modify the original array
                // without reference support, so we'll return a new array
                let arr = self.eval_expr(arr_expr.clone())?;
                match arr {
                    Value::Array(mut elements) => {
                        elements.push(value);
                        Ok(Value::Array(elements))
                    }
                    _ => Err(String::from("push() requires an array")),
                }
            }
            "sum" => {
                if args.len() != 1 {
                    return Err(format!("sum() takes 1 argument, got {}", args.len()));
                }
                let arr = self.eval_expr(args[0].clone())?;
                match arr {
                    Value::Array(elements) => {
                        let mut total = 0i64;
                        for elem in elements {
                            if let Some(n) = elem.to_number() {
                                total += n;
                            } else {
                                return Err(String::from("sum() requires array of numbers"));
                            }
                        }
                        Ok(Value::Number(total))
                    }
                    _ => Err(String::from("sum() requires an array")),
                }
            }
            _ => {
                // User-defined function
                let func = self.functions.get(&name)
                    .ok_or(format!("Undefined function: {}", name))?
                    .clone();

                if args.len() != func.params.len() {
                    return Err(format!("Function {} expects {} arguments, got {}",
                        name, func.params.len(), args.len()));
                }

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }

                // Create new scope
                let mut new_scope = BTreeMap::new();
                for (param, value) in func.params.iter().zip(arg_values.iter()) {
                    new_scope.insert(param.clone(), value.clone());
                }
                self.locals.push(new_scope);

                // Execute function body
                for stmt in func.body {
                    self.execute_stmt(stmt)?;
                    if self.return_value.is_some() {
                        break;
                    }
                }

                // Pop scope
                self.locals.pop();

                // Get return value
                let result = self.return_value.take().unwrap_or(Value::Null);
                Ok(result)
            }
        }
    }

    fn get_var(&self, name: &str) -> Result<Value, String> {
        // Check locals (from innermost to outermost)
        for scope in self.locals.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // Check globals
        self.globals
            .get(name)
            .cloned()
            .ok_or(format!("Undefined variable: {}", name))
    }

    fn set_var(&mut self, name: String, value: Value) {
        // Set in current scope if exists, otherwise global
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }
}
