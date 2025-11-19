pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod value;

pub use interpreter::Interpreter;
pub use value::Value;

/// HAL Script - A simple, powerful embedded scripting language
///
/// Features:
/// - Dynamic typing
/// - Functions
/// - Conditionals (if/else)
/// - Loops (for, while)
/// - Arrays
/// - String interpolation
/// - System calls
///
/// Example:
/// ```
/// fn fib(n) {
///     if n < 2 {
///         return n
///     }
///     return fib(n-1) + fib(n-2)
/// }
///
/// print fib(10)
/// ```
