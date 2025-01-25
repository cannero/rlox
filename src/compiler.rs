use crate::scanner::{Scanner, TokenType};

pub struct Compiler {
}

impl Compiler {
    pub fn new() -> Self {
        Self{}
    }

    pub fn compile(&mut self, source: String) {
        let mut scanner = Scanner::new(source);
        let mut line = -1;
        loop {
            match scanner.scan_token(){
                Ok(token) => {
                    if token.line != line {
                        print!("{:>4}", token.line);
                        line = token.line;
                    } else {
                        print!("   |");
                    }

                    println!("{token:?}");

                    if token.token_type == TokenType::Eof {
                        break;
                    }
                }
                Err(err_token) => println!("error {}", err_token.message),
            }
        }
    }
}
