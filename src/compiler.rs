use std::{collections::HashMap, sync::LazyLock};

use crate::{chunk::Chunk, op_code::OpCode, scanner::{ErrorToken, Scanner, Token, TokenType}, value::Value};

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

type ParseFn = fn(&mut Compiler, bool);

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
    (TokenType::Identifier, ParseRule::prefix(Compiler::variable)),
    (TokenType::String, ParseRule::prefix(Compiler::string)),
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

    fn no_panic(&mut self) {
        self.panic_mode = false;
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
        while !self.match_it(TokenType::Eof) {
            self.declaration();
        }

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

    fn declaration(&mut self) {
        if self.match_it(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_it(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_it(TokenType::Equal) {
            self.expression();
        } else {
            self.write(OpCode::Nil);
        }

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration");

        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression");
        self.write(OpCode::Pop);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.write(OpCode::Print);
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.parser.matches(token_type) {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn match_it(&mut self, token_type: TokenType) -> bool {
        if !self.parser.matches(token_type) {
            return false;
        }

        self.advance();
        true
    }

    fn end_compiler(&mut self) {
        self.write(OpCode::Return);
    }

    fn binary(&mut self, _can_assign: bool) {
        if self.debug {
            println!("binary");
        }

        let operator_type = self.parser.previous.token_type;
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.next_level());

        match operator_type {
            TokenType::BangEqual => self.write2(OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual => self.write(OpCode::Equal),
            TokenType::Greater => self.write(OpCode::Greater),
            TokenType::GreaterEqual => self.write2(OpCode::Less, OpCode::Not),
            TokenType::Less => self.write(OpCode::Less),
            TokenType::LessEqual => self.write2(OpCode::Greater, OpCode::Not),
            TokenType::Plus => self.write(OpCode::Add),
            TokenType::Minus => self.write(OpCode::Subtract),
            TokenType::Star => self.write(OpCode::Multiply),
            TokenType::Slash => self.write(OpCode::Divide),
            _ => panic!("wrong token type in binary {:?}", operator_type),
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        let token_type = self.parser.previous.token_type;
        
        match token_type {
            TokenType::False => self.write(OpCode::Bool(false)),
            TokenType::Nil => self.write(OpCode::Nil),
            TokenType::True => self.write(OpCode::Bool(true)),
            _ => panic!("wrong token type in literal {:?}", token_type),
        }
    }

    fn number(&mut self, _can_assign: bool) {
        let num = self.scanner.lexeme(&self.parser.previous).parse::<f32>()
            .expect("not a valid number");
        self.write(OpCode::Constant(num));
    }

    fn string(&mut self, _can_assign: bool) {
        // todo: maybe use range in the OpCode, then the vm can get the string
        // from the compiler.
        let string = self.lexeme_string(&self.parser.previous);
        self.write(OpCode::String(string));
    }

    fn variable(&mut self, can_assign: bool) {
        let name = self.lexeme(&self.parser.previous);
        self.named_variable(name, can_assign);
    }

    fn named_variable(&mut self, name: String, can_assign: bool) {
        if can_assign && self.match_it(TokenType::Equal) {
            self.expression();
            self.write(OpCode::SetGlobal(name));
        } else {
            self.write(OpCode::GetGlobal(name));
        }
    }

    fn grouping(&mut self, _can_assign: bool) {
        if self.debug {
            println!("grouping");
        }
        self.expression();
        self.consume(TokenType::RightParen, "expected ')' after expression");
        if self.debug {
            println!("grouping end");
        }
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = self.parser.previous.token_type;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Bang => self.write(OpCode::Not),
            TokenType::Minus => self.write(OpCode::Negate),
            _ => panic!("wrong token type in unary {:?}", operator_type),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        if self.debug {
            println!("parse {precedence:?}");
        }

        self.advance();
        let can_assign = precedence <= Precedence::Assignment;
        // todo: move get_rule to parser for previous and current token
        let prefix_rule = self.get_rule(self.parser.previous.token_type).prefix;

        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self, can_assign);
        } else {
            println!("{:?}", self.parser.previous.token_type);
            self.error("Expect expression");
            return;
        }

        while precedence <= self.get_rule(self.parser.current.token_type).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.parser.previous.token_type).infix.expect("infix must be defined");
            infix_rule(self, can_assign);
        }

        if can_assign && self.match_it(TokenType::Equal) {
            self.error("Invalid assignment");
        }
    }

    fn write(&mut self, code: OpCode) {
        let line = self.parser.previous.line;
        self.chunk.write(code, line);
    }

    fn write2(&mut self, code1: OpCode, code2: OpCode) {
        self.write(code1);
        self.write(code2);
    }

    fn lexeme_string(&self, token: &Token) -> String {
        self.scanner.lexeme_string(token)
    }

    fn lexeme(&self, token: &Token) -> String {
        self.scanner.lexeme(token)
    }

    fn synchronize(&mut self) {
        self.parser.no_panic();
        while self.parser.current.token_type != TokenType::Eof {
            if self.parser.previous.token_type == TokenType::Semicolon {
                return;
            }

            match self.parser.current.token_type {
                TokenType::Class | TokenType::Fun | TokenType::Var |
                TokenType::For | TokenType::If | TokenType::While |
                TokenType::Print | TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> String {
        self.consume(TokenType::Identifier, error_message);
        self.lexeme(&self.parser.previous)
    }

    fn define_variable(&mut self, id: String) {
        self.write(OpCode::DefineGlobal(id));
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
