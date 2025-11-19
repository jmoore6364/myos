#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_numbers() {
        let mut lexer = Lexer::new("42 -17 0");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Number(42));
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Number(17));
        assert_eq!(tokens[3], Token::Number(0));
    }

    #[test]
    fn test_tokenize_strings() {
        let mut lexer = Lexer::new(r#""hello" "world""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::String("hello".into()));
        assert_eq!(tokens[1], Token::String("world".into()));
    }

    #[test]
    fn test_tokenize_keywords() {
        let mut lexer = Lexer::new("let fn if else while for return print");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Fn);
        assert_eq!(tokens[2], Token::If);
        assert_eq!(tokens[3], Token::Else);
        assert_eq!(tokens[4], Token::While);
        assert_eq!(tokens[5], Token::For);
        assert_eq!(tokens[6], Token::Return);
        assert_eq!(tokens[7], Token::Print);
    }

    #[test]
    fn test_tokenize_operators() {
        let mut lexer = Lexer::new("+ - * / % == != < > <= >= && ||");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Slash);
        assert_eq!(tokens[4], Token::Percent);
        assert_eq!(tokens[5], Token::EqEq);
        assert_eq!(tokens[6], Token::NotEq);
        assert_eq!(tokens[7], Token::Lt);
        assert_eq!(tokens[8], Token::Gt);
        assert_eq!(tokens[9], Token::LtEq);
        assert_eq!(tokens[10], Token::GtEq);
        assert_eq!(tokens[11], Token::And);
        assert_eq!(tokens[12], Token::Or);
    }

    #[test]
    fn test_tokenize_identifiers() {
        let mut lexer = Lexer::new("foo bar_baz x123");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Ident("foo".into()));
        assert_eq!(tokens[1], Token::Ident("bar_baz".into()));
        assert_eq!(tokens[2], Token::Ident("x123".into()));
    }

    #[test]
    fn test_tokenize_comments() {
        let mut lexer = Lexer::new("x = 10 # this is a comment\ny = 20");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Ident("x".into()));
        assert_eq!(tokens[1], Token::Eq);
        assert_eq!(tokens[2], Token::Number(10));
        assert_eq!(tokens[3], Token::Newline);
        assert_eq!(tokens[4], Token::Ident("y".into()));
    }

    #[test]
    fn test_string_escapes() {
        let mut lexer = Lexer::new(r#""hello\nworld\t!""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::String("hello\nworld\t!".into()));
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""hello"#);
        let result = lexer.tokenize();

        assert!(result.is_err());
    }
}
