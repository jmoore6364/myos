use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use super::lexer::Token;
use super::value::Value;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Ident(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Call(String, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Array(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    Assign(String, Expr),
    Print(Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    For(String, Expr, Expr, Vec<Stmt>),
    Return(Option<Expr>),
    Function(String, Vec<String>, Vec<Stmt>),
    Break,
    Continue,
    Expr(Expr),
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }
            statements.push(self.statement()?);
            self.skip_newlines();
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Token::Let => self.let_stmt(),
            Token::Print => self.print_stmt(),
            Token::If => self.if_stmt(),
            Token::While => self.while_stmt(),
            Token::For => self.for_stmt(),
            Token::Return => self.return_stmt(),
            Token::Break => self.break_stmt(),
            Token::Continue => self.continue_stmt(),
            Token::Fn => self.function_stmt(),
            Token::Ident(_) => {
                if self.peek_ahead(1) == Some(&Token::Eq) {
                    self.assign_stmt()
                } else {
                    Ok(Stmt::Expr(self.expression()?))
                }
            }
            _ => Ok(Stmt::Expr(self.expression()?)),
        }
    }

    fn let_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Let)?;
        let name = self.expect_ident()?;
        self.expect(Token::Eq)?;
        let expr = self.expression()?;
        Ok(Stmt::Let(name, expr))
    }

    fn assign_stmt(&mut self) -> Result<Stmt, String> {
        let name = self.expect_ident()?;
        self.expect(Token::Eq)?;
        let expr = self.expression()?;
        Ok(Stmt::Assign(name, expr))
    }

    fn print_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Print)?;
        let expr = self.expression()?;
        Ok(Stmt::Print(expr))
    }

    fn if_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(Token::If)?;
        let condition = self.expression()?;
        self.expect(Token::LBrace)?;
        let then_branch = self.block()?;
        self.expect(Token::RBrace)?;

        let else_branch = if self.peek() == &Token::Else {
            self.advance();
            self.expect(Token::LBrace)?;
            let branch = self.block()?;
            self.expect(Token::RBrace)?;
            Some(branch)
        } else {
            None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn while_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(Token::While)?;
        let condition = self.expression()?;
        self.expect(Token::LBrace)?;
        let body = self.block()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::While(condition, body))
    }

    fn for_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(Token::For)?;
        let var = self.expect_ident()?;
        self.expect(Token::In)?;
        let start = self.expression()?;
        self.expect(Token::DotDot)?;
        let end = self.expression()?;
        self.expect(Token::LBrace)?;
        let body = self.block()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::For(var, start, end, body))
    }

    fn return_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Return)?;
        if self.peek() == &Token::Newline || self.peek() == &Token::RBrace {
            Ok(Stmt::Return(None))
        } else {
            Ok(Stmt::Return(Some(self.expression()?)))
        }
    }

    fn break_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Break)?;
        Ok(Stmt::Break)
    }

    fn continue_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Continue)?;
        Ok(Stmt::Continue)
    }

    fn function_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Fn)?;
        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;

        let mut params = Vec::new();
        if self.peek() != &Token::RParen {
            params.push(self.expect_ident()?);
            while self.peek() == &Token::Comma {
                self.advance();
                params.push(self.expect_ident()?);
            }
        }

        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let body = self.block()?;
        self.expect(Token::RBrace)?;

        Ok(Stmt::Function(name, params, body))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();

        while self.peek() != &Token::RBrace && !self.is_at_end() {
            self.skip_newlines();
            if self.peek() == &Token::RBrace {
                break;
            }
            statements.push(self.statement()?);
            self.skip_newlines();
        }

        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.logical_and()?;

        while self.peek() == &Token::Or {
            self.advance();
            let right = self.logical_and()?;
            expr = Expr::Binary(Box::new(expr), BinOp::Or, Box::new(right));
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.peek() == &Token::And {
            self.advance();
            let right = self.equality()?;
            expr = Expr::Binary(Box::new(expr), BinOp::And, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while matches!(self.peek(), Token::EqEq | Token::NotEq) {
            let op = match self.advance() {
                Token::EqEq => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while matches!(self.peek(), Token::Lt | Token::Gt | Token::LtEq | Token::GtEq) {
            let op = match self.advance() {
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = match self.advance() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while matches!(self.peek(), Token::Star | Token::Slash | Token::Percent) {
            let op = match self.advance() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Token::Minus | Token::Not) {
            let op = match self.advance() {
                Token::Minus => UnOp::Neg,
                Token::Not => UnOp::Not,
                _ => unreachable!(),
            };
            Ok(Expr::Unary(op, Box::new(self.unary()?)))
        } else {
            self.postfix()
        }
    }

    fn postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            match self.peek() {
                Token::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek() != &Token::RParen {
                        args.push(self.expression()?);
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.expression()?);
                        }
                    }
                    self.expect(Token::RParen)?;

                    if let Expr::Ident(name) = expr {
                        expr = Expr::Call(name, args);
                    } else {
                        return Err(String::from("Can only call functions"));
                    }
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.expression()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr::Index(Box::new(expr), Box::new(index));
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Token::Number(n) => Ok(Expr::Literal(Value::Number(n))),
            Token::String(s) => Ok(Expr::Literal(Value::String(s))),
            Token::True => Ok(Expr::Literal(Value::Bool(true))),
            Token::False => Ok(Expr::Literal(Value::Bool(false))),
            Token::Null => Ok(Expr::Literal(Value::Null)),
            Token::Ident(name) => Ok(Expr::Ident(name)),
            Token::LParen => {
                let expr = self.expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                let mut elements = Vec::new();
                if self.peek() != &Token::RBracket {
                    elements.push(self.expression()?);
                    while self.peek() == &Token::Comma {
                        self.advance();
                        elements.push(self.expression()?);
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::Array(elements))
            }
            token => Err(format!("Unexpected token: {:?}", token)),
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.peek() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.peek()))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(name) => Ok(name),
            token => Err(format!("Expected identifier, got {:?}", token)),
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position + offset)
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        if self.position < self.tokens.len() - 1 {
            self.position += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.peek() == &Token::Eof
    }
}
