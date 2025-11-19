use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(i64),
    String(String),
    Array(Vec<Value>),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
        }
    }

    pub fn to_number(&self) -> Option<i64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Null => String::from("null"),
            Value::Bool(b) => format!("{}", b),
            Value::Number(n) => format!("{}", n),
            Value::String(s) => s.clone(),
            Value::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => {
                let mut result = a.clone();
                result.push_str(b);
                Ok(Value::String(result))
            }
            (Value::String(a), other) => {
                let mut result = a.clone();
                result.push_str(&other.to_string());
                Ok(Value::String(result))
            }
            (this, Value::String(b)) => {
                let mut result = this.to_string();
                result.push_str(b);
                Ok(Value::String(result))
            }
            _ => Err(format!("Cannot add {:?} and {:?}", self, other)),
        }
    }

    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Ok(Value::Number(a - b)),
            _ => Err(format!("Cannot subtract {:?} from {:?}", other, self)),
        }
    }

    pub fn mul(&self, other: &Value) -> Result<Value, String> {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Ok(Value::Number(a * b)),
            _ => Err(format!("Cannot multiply {:?} and {:?}", self, other)),
        }
    }

    pub fn div(&self, other: &Value) -> Result<Value, String> {
        match (self.to_number(), other.to_number()) {
            (Some(_), Some(0)) => Err(String::from("Division by zero")),
            (Some(a), Some(b)) => Ok(Value::Number(a / b)),
            _ => Err(format!("Cannot divide {:?} by {:?}", self, other)),
        }
    }

    pub fn compare(&self, other: &Value) -> Option<core::cmp::Ordering> {
        use core::cmp::Ordering;
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(a.cmp(b)),
            (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
            (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
