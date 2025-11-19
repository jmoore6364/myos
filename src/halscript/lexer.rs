use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(i64),
    String(String),
    Ident(String),

    // Keywords
    Let,
    Fn,
    If,
    Else,
    For,
    While,
    In,
    Return,
    Print,
    True,
    False,
    Null,
    Break,
    Continue,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Not,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Dot,
    DotDot,
    Newline,

    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();

            if self.is_at_end() {
                break;
            }

            // Skip comments
            if self.peek() == Some('#') {
                while self.peek() != Some('\n') && !self.is_at_end() {
                    self.advance();
                }
                continue;
            }

            tokens.push(self.next_token()?);
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let ch = self.advance().ok_or("Unexpected end of input")?;

        let token = match ch {
            '\n' => Token::Newline,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    Token::DotDot
                } else {
                    Token::Dot
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::EqEq
                } else {
                    Token::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::NotEq
                } else {
                    Token::Not
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::LtEq
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GtEq
                } else {
                    Token::Gt
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    Token::And
                } else {
                    return Err(String::from("Expected '&&'"));
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    Token::Or
                } else {
                    return Err(String::from("Expected '||'"));
                }
            }
            '"' => self.read_string()?,
            c if c.is_ascii_digit() => {
                self.position -= 1;
                self.read_number()?
            }
            c if c.is_alphabetic() || c == '_' => {
                self.position -= 1;
                self.read_ident()
            }
            c => return Err(format!("Unexpected character: '{}'", c)),
        };

        Ok(token)
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let mut s = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                return Ok(Token::String(s));
            }

            if ch == '\\' {
                self.advance();
                match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some(c) => s.push(c),
                    None => return Err(String::from("Unterminated string")),
                }
            } else {
                s.push(ch);
                self.advance();
            }
        }

        Err(String::from("Unterminated string"))
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let mut num = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        num.parse::<i64>()
            .map(Token::Number)
            .map_err(|_| format!("Invalid number: {}", num))
    }

    fn read_ident(&mut self) -> Token {
        let mut ident = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        match ident.as_str() {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "in" => Token::In,
            "return" => Token::Return,
            "print" => Token::Print,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "break" => Token::Break,
            "continue" => Token::Continue,
            _ => Token::Ident(ident),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let ch = self.input[self.position];
            self.position += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
#[path = "lexer_tests.rs"]
mod tests;
