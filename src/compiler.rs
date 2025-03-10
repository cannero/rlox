use std::{collections::HashMap, sync::LazyLock};

use crate::{chunk::Chunk, op_code::OpCode, scanner::{ErrorToken, Scanner, Token, TokenType}};

pub type CompileResult = Result<Chunk, ()>;

#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment,  // =
    Or,          // or
    And,         // and
    Equality,    // == !=
    Comparison,  // < > <= >=
    Term,        // + -
    Factor,      // * /
    Unary,       // ! -
    Call,        // . ()
    Primary,
}

impl Precedence {
    fn next_level(&self) -> Self {
        match self {
            Precedence::None => Self::Assignment,
            Precedence::Assignment => Self::Or,
            Precedence::Or => Self::And,
            Precedence::And => Self::Equality,
            Precedence::Equality => Self::Comparison,
            Precedence::Comparison => Self::Term,
            Precedence::Term => Self::Factor,
            Precedence::Factor => Self::Unary,
            Precedence::Unary => Self::Call,
            Precedence::Call => Self::Primary,
            Precedence::Primary => panic!("no next precedence level"),
        }
    }
}

type ParseFn = fn(&mut Compiler);

struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl ParseRule {
    const fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        Self { prefix, infix, precedence, }
    }

    const fn infix(infix: ParseFn, precedence: Precedence) -> Self {
        Self {prefix: None, infix: Some(infix), precedence, }
    }

    const fn prefix(prefix: ParseFn) -> Self {
        Self { prefix: Some(prefix), infix: None, precedence: Precedence::None }
    }

    const fn undef() -> Self {
        Self { prefix: None, infix: None, precedence: Precedence::None, }
    }
}

static RULES: LazyLock<HashMap<TokenType, ParseRule>> = LazyLock::new(|| HashMap::from([
    (TokenType::LeftParen, ParseRule::new(Some(Compiler::grouping), None, Precedence::None)),
    (TokenType::RightParen, ParseRule::undef()),
    (TokenType::LeftBrace, ParseRule::undef()),
    (TokenType::RightBrace, ParseRule::undef()),
    (TokenType::Comma, ParseRule::undef()),
    (TokenType::Dot, ParseRule::undef()),
    (TokenType::Minus, ParseRule::new(Some(Compiler::unary), Some(Compiler::binary), Precedence::Term)),
    (TokenType::Plus, ParseRule::infix(Compiler::binary, Precedence::Term)),
    (TokenType::Semicolon, ParseRule::undef()),
    (TokenType::Slash, ParseRule::infix(Compiler::binary, Precedence::Factor)),
    (TokenType::Star, ParseRule::infix(Compiler::binary, Precedence::Factor)),
    (TokenType::Bang, ParseRule::prefix(Compiler::unary)),
    (TokenType::BangEqual, ParseRule::infix(Compiler::binary, Precedence::Equality)),
    (TokenType::Equal, ParseRule::undef()),
    (TokenType::EqualEqual, ParseRule::infix(Compiler::binary, Precedence::Equality)),
    (TokenType::Greater, ParseRule::infix(Compiler::binary, Precedence::Comparison)),
    (TokenType::GreaterEqual, ParseRule::infix(Compiler::binary, Precedence::Comparison)),
    (TokenType::Less, ParseRule::infix(Compiler::binary, Precedence::Comparison)),
    (TokenType::LessEqual, ParseRule::infix(Compiler::binary, Precedence::Comparison)),
    (TokenType::Identifier, ParseRule::undef()),
    (TokenType::String, ParseRule::undef()),
    (TokenType::Number, ParseRule::prefix(Compiler::number)),
    (TokenType::And, ParseRule::undef()),
    (TokenType::Class, ParseRule::undef()),
    (TokenType::Else, ParseRule::undef()),
    (TokenType::False, ParseRule::prefix(Compiler::literal)),
    (TokenType::For, ParseRule::undef()),
    (TokenType::Fun, ParseRule::undef()),
    (TokenType::If, ParseRule::undef()),
    (TokenType::Nil, ParseRule::prefix(Compiler::literal)),
    (TokenType::Or, ParseRule::undef()),
    (TokenType::Print, ParseRule::undef()),
    (TokenType::Return, ParseRule::undef()),
    (TokenType::Super, ParseRule::undef()),
    (TokenType::This, ParseRule::undef()),
    (TokenType::True, ParseRule::prefix(Compiler::literal)),
    (TokenType::Var, ParseRule::undef()),
    (TokenType::While, ParseRule::undef()),
    (TokenType::Eof, ParseRule::undef()),
]));

fn get_rule(token_type: TokenType) -> &'static ParseRule {
    &RULES.get(&token_type).expect("rule must exist")
}

struct Parser {
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    fn new() -> Self {
        Self {
            current: Token { token_type: TokenType::Eof, line: 0, start: 0, length: 0 },
            previous: Token { token_type: TokenType::Eof, line: 0, start: 0, length: 0 },
            had_error: false,
            panic_mode: false,
        }
    }

    fn set_token(&mut self, token: Token) {
        self.previous = std::mem::replace(&mut self.current, token);
    }

    fn had_error(&mut self) {
        self.had_error = true;
    }

    fn panic(&mut self) {
        self.panic_mode = true;
    }

    fn matches(&self, token_type: TokenType) -> bool {
        self.current.token_type == token_type
    }
}

pub fn compile(source: String, debug: bool) -> CompileResult {
    let mut compiler = Compiler::new(source, debug);
    if compiler.compile() {
        Ok(compiler.chunk)
    } else {
        Err(())
    }
}

struct Compiler {
    scanner: Scanner,
    parser: Parser,
    chunk: Chunk,
    debug: bool,
}

impl Compiler {
    fn new(source: String, debug: bool) -> Self {
        Self { scanner: Scanner::new(&source),
               parser: Parser::new(),
               chunk: Chunk::new(),
               debug,
        }
    }

    fn compile(&mut self) -> bool {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        
        self.end_compiler();
        !self.parser.had_error
    }

    fn advance(&mut self) {
        loop {
            match self.scanner.scan_token() {
                Ok(token) => {
                    self.parser.set_token(token);
                    break;
                },
                Err(err_token) => self.show_error(err_token, "error during advance"),
            }
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.parser.matches(token_type) {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }


    fn end_compiler(&mut self) {
        self.chunk.write(OpCode::Return, 9999);
    }

    fn binary(&mut self) {
        if self.debug {
            println!("binary");
        }

        let operator_type = self.parser.previous.token_type;
        let line = self.parser.previous.line;
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.next_level());

        match operator_type {
            TokenType::BangEqual => self.chunk.write2(OpCode::Equal, OpCode::Not, line),
            TokenType::EqualEqual => self.chunk.write(OpCode::Equal, line),
            TokenType::Greater => self.chunk.write(OpCode::Greater, line),
            TokenType::GreaterEqual => self.chunk.write2(OpCode::Less, OpCode::Not, line),
            TokenType::Less => self.chunk.write(OpCode::Less, line),
            TokenType::LessEqual => self.chunk.write2(OpCode::Greater, OpCode::Not, line),
            TokenType::Plus => self.chunk.write(OpCode::Add, line),
            TokenType::Minus => self.chunk.write(OpCode::Subtract, line),
            TokenType::Star => self.chunk.write(OpCode::Multiply, line),
            TokenType::Slash => self.chunk.write(OpCode::Divide, line),
            _ => panic!("wrong token type in binary {:?}", operator_type),
        }
    }

    fn literal(&mut self) {
        let token_type = self.parser.previous.token_type;
        let line = self.parser.previous.line;
        
        match token_type {
            TokenType::False => self.chunk.write(OpCode::Bool(false), line),
            TokenType::Nil => self.chunk.write(OpCode::Nil, line),
            TokenType::True => self.chunk.write(OpCode::Bool(true), line),
            _ => panic!("wrong token type in literal {:?}", token_type),
        }
    }

    fn number(&mut self) {
        let num = self.scanner.lexeme(&self.parser.previous).parse::<f32>()
            .expect("not a valid number");
        self.chunk.write(OpCode::Constant(num), self.parser.previous.line);
    }

    fn grouping(&mut self) {
        if self.debug {
            println!("grouping");
        }
        self.expression();
        self.consume(TokenType::RightParen, "expected ')' after expression");
        if self.debug {
            println!("grouping end");
        }
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.token_type;
        let line = self.parser.previous.line;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Bang => self.chunk.write(OpCode::Not, line),
            TokenType::Minus => self.chunk.write(OpCode::Negate, line),
            _ => panic!("wrong token type in unary {:?}", operator_type),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        if self.debug {
            println!("parse {precedence:?}");
        }

        self.advance();
        // todo: move get_rule to parser for previous and current token
        let prefix_rule = self.get_rule(self.parser.previous.token_type).prefix;

        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self);
        } else {
            println!("{:?}", self.parser.previous.token_type);
            self.error("Expect expression");
            return;
        }

        while precedence <= self.get_rule(self.parser.current.token_type).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.parser.previous.token_type).infix.expect("infix must be defined");
            infix_rule(self);
        }
    }

    fn get_rule(&self, operator_type: TokenType) -> &ParseRule {
        get_rule(operator_type)
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.parser.current.clone(), message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.parser.previous.clone(), message);
    }

    fn error_at(&mut self, token: Token, message: &str) {
        if self.parser.panic_mode {
            return;
        }

        self.parser.panic();
        eprint!("[line {}] Error", token.line);

        if token.token_type == TokenType::Eof {
            eprint!(" at end");
        } else {
            eprint!(" at {} ({:?})", self.scanner.get_lexeme(&token), token.token_type);
        }

        eprintln!(": {message}");
        self.parser.had_error();
    }

    fn show_error(&mut self, token: ErrorToken, message: &str) {
        if self.parser.panic_mode {
            return;
        }

        self.parser.panic();
        eprint!("[line {}] Error", token.line);
        eprint!(" at {}", self.scanner.get_lexeme_error(&token));
        eprintln!(": {message}");
        self.parser.had_error();
    }
}
