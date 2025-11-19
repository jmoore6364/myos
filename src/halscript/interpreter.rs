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
    loaded_modules: alloc::collections::BTreeSet<String>,
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
            loaded_modules: alloc::collections::BTreeSet::new(),
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
            Stmt::Import(path) => {
                // Check if already loaded
                if self.loaded_modules.contains(&path) {
                    return Ok(());
                }

                // Read module from VFS
                let vfs = crate::vfs::VFS.lock();
                let module_code = vfs.read_file(&path)
                    .map_err(|e| format!("Failed to import {}: {}", path, e))?;
                drop(vfs);  // Release the lock

                // Parse the module
                let mut lexer = super::lexer::Lexer::new(&module_code);
                let tokens = lexer.tokenize()
                    .map_err(|e| format!("Failed to tokenize {}: {}", path, e))?;
                let mut parser = super::parser::Parser::new(tokens);
                let statements = parser.parse()
                    .map_err(|e| format!("Failed to parse {}: {}", path, e))?;

                // Mark as loaded before executing (prevent circular imports)
                self.loaded_modules.insert(path.clone());

                // Execute the module
                for stmt in statements {
                    self.execute_stmt(stmt)?;
                }

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
                let container = self.eval_expr(*arr_expr)?;
                let index = self.eval_expr(*index_expr)?;

                match container {
                    Value::Array(elements) => {
                        if let Some(idx) = index.to_number() {
                            if idx >= 0 && (idx as usize) < elements.len() {
                                Ok(elements[idx as usize].clone())
                            } else {
                                Err(format!("Index {} out of bounds", idx))
                            }
                        } else {
                            Err(String::from("Array index must be a number"))
                        }
                    }
                    Value::Map(map) => {
                        if let Value::String(key) = index {
                            map.get(&key)
                                .cloned()
                                .ok_or(format!("Key '{}' not found in map", key))
                        } else {
                            Err(String::from("Map index must be a string"))
                        }
                    }
                    _ => Err(String::from("Can only index arrays and maps")),
                }
            }
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_expr(elem)?);
                }
                Ok(Value::Array(values))
            }
            Expr::Map(entries) => {
                let mut map = alloc::collections::BTreeMap::new();
                for (key, value_expr) in entries {
                    let value = self.eval_expr(value_expr)?;
                    map.insert(key, value);
                }
                Ok(Value::Map(map))
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
            "split" => {
                if args.len() != 2 {
                    return Err(format!("split() takes 2 arguments, got {}", args.len()));
                }
                let string = self.eval_expr(args[0].clone())?;
                let delimiter = self.eval_expr(args[1].clone())?;

                match (string, delimiter) {
                    (Value::String(s), Value::String(d)) => {
                        let parts: Vec<Value> = s.split(&d as &str)
                            .map(|p| Value::String(String::from(p)))
                            .collect();
                        Ok(Value::Array(parts))
                    }
                    _ => Err(String::from("split() requires two strings")),
                }
            }
            "trim" => {
                if args.len() != 1 {
                    return Err(format!("trim() takes 1 argument, got {}", args.len()));
                }
                let value = self.eval_expr(args[0].clone())?;
                match value {
                    Value::String(s) => Ok(Value::String(String::from(s.trim()))),
                    _ => Err(String::from("trim() requires a string")),
                }
            }
            "upper" => {
                if args.len() != 1 {
                    return Err(format!("upper() takes 1 argument, got {}", args.len()));
                }
                let value = self.eval_expr(args[0].clone())?;
                match value {
                    Value::String(s) => {
                        let upper: String = s.chars().map(|c| {
                            if c.is_ascii_lowercase() {
                                ((c as u8) - 32) as char
                            } else {
                                c
                            }
                        }).collect();
                        Ok(Value::String(upper))
                    }
                    _ => Err(String::from("upper() requires a string")),
                }
            }
            "lower" => {
                if args.len() != 1 {
                    return Err(format!("lower() takes 1 argument, got {}", args.len()));
                }
                let value = self.eval_expr(args[0].clone())?;
                match value {
                    Value::String(s) => {
                        let lower: String = s.chars().map(|c| {
                            if c.is_ascii_uppercase() {
                                ((c as u8) + 32) as char
                            } else {
                                c
                            }
                        }).collect();
                        Ok(Value::String(lower))
                    }
                    _ => Err(String::from("lower() requires a string")),
                }
            }
            "replace" => {
                if args.len() != 3 {
                    return Err(format!("replace() takes 3 arguments, got {}", args.len()));
                }
                let string = self.eval_expr(args[0].clone())?;
                let old = self.eval_expr(args[1].clone())?;
                let new = self.eval_expr(args[2].clone())?;

                match (string, old, new) {
                    (Value::String(s), Value::String(o), Value::String(n)) => {
                        Ok(Value::String(s.replace(&o as &str, &n as &str)))
                    }
                    _ => Err(String::from("replace() requires three strings")),
                }
            }
            "starts_with" => {
                if args.len() != 2 {
                    return Err(format!("starts_with() takes 2 arguments, got {}", args.len()));
                }
                let string = self.eval_expr(args[0].clone())?;
                let prefix = self.eval_expr(args[1].clone())?;

                match (string, prefix) {
                    (Value::String(s), Value::String(p)) => {
                        Ok(Value::Bool(s.starts_with(&p as &str)))
                    }
                    _ => Err(String::from("starts_with() requires two strings")),
                }
            }
            "ends_with" => {
                if args.len() != 2 {
                    return Err(format!("ends_with() takes 2 arguments, got {}", args.len()));
                }
                let string = self.eval_expr(args[0].clone())?;
                let suffix = self.eval_expr(args[1].clone())?;

                match (string, suffix) {
                    (Value::String(s), Value::String(suf)) => {
                        Ok(Value::Bool(s.ends_with(&suf as &str)))
                    }
                    _ => Err(String::from("ends_with() requires two strings")),
                }
            }
            "substring" => {
                if args.len() != 3 {
                    return Err(format!("substring() takes 3 arguments, got {}", args.len()));
                }
                let string = self.eval_expr(args[0].clone())?;
                let start = self.eval_expr(args[1].clone())?.to_number()
                    .ok_or("substring() start index must be a number")?;
                let end = self.eval_expr(args[2].clone())?.to_number()
                    .ok_or("substring() end index must be a number")?;

                match string {
                    Value::String(s) => {
                        if start < 0 || end < 0 || start > end {
                            return Err(String::from("Invalid substring indices"));
                        }
                        let start = start as usize;
                        let end = end as usize;
                        if end > s.len() {
                            return Err(String::from("substring() index out of bounds"));
                        }
                        Ok(Value::String(String::from(&s[start..end])))
                    }
                    _ => Err(String::from("substring() requires a string")),
                }
            }
            "pop" => {
                if args.len() != 1 {
                    return Err(format!("pop() takes 1 argument, got {}", args.len()));
                }
                let arr = self.eval_expr(args[0].clone())?;
                match arr {
                    Value::Array(mut elements) => {
                        if elements.is_empty() {
                            Err(String::from("Cannot pop from empty array"))
                        } else {
                            let last = elements.pop().unwrap();
                            // Return array with last element (we return the popped value)
                            Ok(last)
                        }
                    }
                    _ => Err(String::from("pop() requires an array")),
                }
            }
            "reverse" => {
                if args.len() != 1 {
                    return Err(format!("reverse() takes 1 argument, got {}", args.len()));
                }
                let arr = self.eval_expr(args[0].clone())?;
                match arr {
                    Value::Array(mut elements) => {
                        elements.reverse();
                        Ok(Value::Array(elements))
                    }
                    _ => Err(String::from("reverse() requires an array")),
                }
            }
            "join" => {
                if args.len() != 2 {
                    return Err(format!("join() takes 2 arguments, got {}", args.len()));
                }
                let arr = self.eval_expr(args[0].clone())?;
                let separator = self.eval_expr(args[1].clone())?;

                match (arr, separator) {
                    (Value::Array(elements), Value::String(sep)) => {
                        let strings: Vec<String> = elements.iter()
                            .map(|v| v.to_string())
                            .collect();
                        Ok(Value::String(strings.join(&sep as &str)))
                    }
                    _ => Err(String::from("join() requires an array and a string separator")),
                }
            }
            "read_file" => {
                if args.len() != 1 {
                    return Err(format!("read_file() takes 1 argument, got {}", args.len()));
                }
                let path = self.eval_expr(args[0].clone())?;

                match path {
                    Value::String(p) => {
                        let vfs = crate::vfs::VFS.lock();
                        match vfs.read_file(&p) {
                            Ok(content) => Ok(Value::String(content)),
                            Err(e) => Err(e),
                        }
                    }
                    _ => Err(String::from("read_file() requires a string path")),
                }
            }
            "write_file" => {
                if args.len() != 2 {
                    return Err(format!("write_file() takes 2 arguments, got {}", args.len()));
                }
                let path = self.eval_expr(args[0].clone())?;
                let content = self.eval_expr(args[1].clone())?;

                match (path, content) {
                    (Value::String(p), Value::String(c)) => {
                        let mut vfs = crate::vfs::VFS.lock();
                        match vfs.write_file(&p, c) {
                            Ok(_) => Ok(Value::Null),
                            Err(e) => Err(e),
                        }
                    }
                    (Value::String(_), _) => Err(String::from("write_file() content must be a string")),
                    _ => Err(String::from("write_file() requires string path and content")),
                }
            }
            "file_exists" => {
                if args.len() != 1 {
                    return Err(format!("file_exists() takes 1 argument, got {}", args.len()));
                }
                let path = self.eval_expr(args[0].clone())?;

                match path {
                    Value::String(p) => {
                        let vfs = crate::vfs::VFS.lock();
                        Ok(Value::Bool(vfs.file_exists(&p)))
                    }
                    _ => Err(String::from("file_exists() requires a string path")),
                }
            }
            "list_dir" => {
                if args.len() != 1 {
                    return Err(format!("list_dir() takes 1 argument, got {}", args.len()));
                }
                let path = self.eval_expr(args[0].clone())?;

                match path {
                    Value::String(p) => {
                        let vfs = crate::vfs::VFS.lock();
                        match vfs.list_directory(&p) {
                            Ok(entries) => {
                                let files: Vec<Value> = entries.iter()
                                    .map(|(name, _, _)| Value::String(name.clone()))
                                    .collect();
                                Ok(Value::Array(files))
                            }
                            Err(e) => Err(e),
                        }
                    }
                    _ => Err(String::from("list_dir() requires a string path")),
                }
            }
            "keys" => {
                if args.len() != 1 {
                    return Err(format!("keys() takes 1 argument, got {}", args.len()));
                }
                let map_val = self.eval_expr(args[0].clone())?;

                match map_val {
                    Value::Map(m) => {
                        let keys: Vec<Value> = m.keys()
                            .map(|k| Value::String(k.clone()))
                            .collect();
                        Ok(Value::Array(keys))
                    }
                    _ => Err(String::from("keys() requires a map")),
                }
            }
            "values" => {
                if args.len() != 1 {
                    return Err(format!("values() takes 1 argument, got {}", args.len()));
                }
                let map_val = self.eval_expr(args[0].clone())?;

                match map_val {
                    Value::Map(m) => {
                        let vals: Vec<Value> = m.values().cloned().collect();
                        Ok(Value::Array(vals))
                    }
                    _ => Err(String::from("values() requires a map")),
                }
            }
            "has_key" => {
                if args.len() != 2 {
                    return Err(format!("has_key() takes 2 arguments, got {}", args.len()));
                }
                let map_val = self.eval_expr(args[0].clone())?;
                let key_val = self.eval_expr(args[1].clone())?;

                match (map_val, key_val) {
                    (Value::Map(m), Value::String(k)) => {
                        Ok(Value::Bool(m.contains_key(&k)))
                    }
                    (Value::Map(_), _) => Err(String::from("has_key() key must be a string")),
                    _ => Err(String::from("has_key() requires a map and a string key")),
                }
            }
            "map_size" => {
                if args.len() != 1 {
                    return Err(format!("map_size() takes 1 argument, got {}", args.len()));
                }
                let map_val = self.eval_expr(args[0].clone())?;

                match map_val {
                    Value::Map(m) => Ok(Value::Number(m.len() as i64)),
                    _ => Err(String::from("map_size() requires a map")),
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
