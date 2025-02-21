#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus,
    Semicolon, Slash, Star,
    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    // Literals.
    Identifier, String, Number,
    // Keywords.
    And, Class, Else, False,
    For, Fun, If, Nil, Or,
    Print, Return, Super, This,
    True, Var, While,

    // handled by extra type: Error,
    Eof,
}

impl From<TokenType> for usize {
    fn from(value: TokenType) -> Self {
        value as usize
    }
}

pub type ScanResult = Result<Token, ErrorToken>;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: i32,
    pub start: usize,
    pub length: usize,
}

#[derive(Debug, PartialEq)]
pub struct ErrorToken {
    pub line: i32,
    pub start: usize,
    pub length: usize,
    pub message: String,
}

pub struct Scanner {
    // vec of chars to be similar to the c implementation
    // but still allow utf-8
    source: Vec<char>,
    line: i32,
    start: usize,
    current: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self { source: source.chars().collect(), line: 1, start: 0, current: 0 }
    }

    pub fn lexeme(&self, token: &Token) -> String {
        self.source[token.start..token.start+token.length].iter().collect()
    }

    pub fn lexeme_string(&self, token: &Token) -> String {
        match token.token_type {
            TokenType::String => self.source[token.start+1..token.start+token.length-1].iter().collect(),
            _ => panic!("lexeme_string called with {:?}", token.token_type),
        }
    }

    pub fn scan_token(&mut self) -> ScanResult {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

        if self.is_alpha(c) {
            return self.identifier();
        }

        if c.is_ascii_digit() {
            return self.number();
        }
        
        match c {
            '(' => return self.make_token(TokenType::LeftParen),
            ')' => return self.make_token(TokenType::RightParen),
            '{' => return self.make_token(TokenType::LeftBrace),
            '}' => return self.make_token(TokenType::RightBrace),
            ';' => return self.make_token(TokenType::Semicolon),
            ',' => return self.make_token(TokenType::Comma),
            '.' => return self.make_token(TokenType::Dot),
            '-' => return self.make_token(TokenType::Minus),
            '+' => return self.make_token(TokenType::Plus),
            '/' => return self.make_token(TokenType::Slash),
            '*' => return self.make_token(TokenType::Star),
            '!' => {
                return if self.match_char('=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                };
            }
            '=' => {
                return if self.match_char('=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                };
            }
            '<' => {
                return if self.match_char('=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                };
            }
            '>' => {
                return if self.match_char('=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                };
            }
            '"' => return self.string(),
            _ => (),
        }

        Err(self.error_token("Unexpected character"))
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            if matches!(c, '\t' | '\r' | ' ') {
                self.advance();
            } else if c == '\n' {
                self.line += 1;
                self.advance();
            } else if c == '/' {
                if self.peek_next() == '/' {
                    while self.peek() != '\n' && !self.is_at_end(){
                        self.advance();
                    }
                }
            } else {
                 break;
            }
        }
    }

    fn string(&mut self) -> ScanResult {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(self.error_token("Undetermined string"));
        }

        self.advance();

        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> ScanResult {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> ScanResult {
        while self.is_alpha(self.peek()) || self.peek().is_ascii_digit() {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    fn make_token(&self, token_type: TokenType) -> ScanResult {
        Ok(Token { token_type, line: self.line, start: self.start, length: self.current - self.start })
    }

    fn error_token(&self, message: &str) -> ErrorToken {
        ErrorToken { message: message.to_string(), line: self.line, start: self.start, length: self.current }
    }

    fn identifier_type(&self) -> TokenType {
        match self.source[self.start] {
            'a' => self.check_keyword(1, "nd", TokenType::And),
            'c' => self.check_keyword(1, "lass", TokenType::Class),
            'e' => self.check_keyword(1, "lse", TokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.source[self.start + 1] {
                        'a' => self.check_keyword(2, "lse", TokenType::False),
                        'o' => self.check_keyword(2, "r", TokenType::For),
                        'u' => self.check_keyword(2, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, "f", TokenType::If),
            'n' => self.check_keyword(1, "il", TokenType::Nil),
            'o' => self.check_keyword(1, "r", TokenType::Or),
            'p' => self.check_keyword(1, "rint", TokenType::Print),
            'r' => self.check_keyword(1, "eturn", TokenType::Return),
            's' => self.check_keyword(1, "uper", TokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.source[self.start + 1] {
                        'h' => self.check_keyword(2, "is", TokenType::This),
                        'r' => self.check_keyword(2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, "ar", TokenType::Var),
            'w' => self.check_keyword(1, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, rest: &str, token_type: TokenType) -> TokenType {
        if self.current - self.start == start + rest.len() &&
           self.source[self.start+start..self.current].iter().collect::<String>() == rest {
               token_type
        } else {
                TokenType::Identifier
        }
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn match_char(&mut self, c: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current] == c {
            self.current += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    pub fn get_lexeme(&self, token: &Token) -> String {
        self.source[token.start..token.start+token.length].iter().collect::<String>()
    }

    pub fn get_lexeme_error(&self, token: &ErrorToken) -> String {
        self.source[token.start..token.start+token.length].iter().collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create(source: &str) -> Scanner {
        Scanner::new(source)
    }

    fn assert_token(result: ScanResult, expected: Token) {
        assert_eq!(result, Ok(expected));
    }
    
    #[test]
    fn test_else_token() {
        let mut target = create("else");
        let res = target.scan_token();
        let expected = Token { token_type: TokenType::Else, line: 1, start: 0, length: 4 };
        assert_token(res, expected);
    }

    #[test]
    fn test_false_token() {
        let mut target = create("false");
        let res = target.scan_token();
        let expected = Token { token_type: TokenType::False, line: 1, start: 0, length: 5 };
        assert_token(res, expected);
    }

    #[test]
    fn test_identifier() {
        let mut target = create("falso");
        let res = target.scan_token();
        let expected = Token { token_type: TokenType::Identifier, line: 1, start: 0, length: 5 };
        assert_token(res, expected);
    }

    #[test]
    fn test_whitespace() {
        let mut target = create(" ");
        let res = target.scan_token();
        let expected = Token { token_type: TokenType::Eof, line: 1, start: 1, length: 0 };
        assert_token(res, expected);
    }

    #[test]
    fn test_invalid_input() {
        let mut target = create("\"str");
        let res = target.scan_token();
        let expected = ErrorToken { message: "Undetermined string".to_string(), line: 1, start: 0, length: 4 };
        assert_eq!(res, Err(expected));
    }
}
