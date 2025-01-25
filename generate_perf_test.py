def generate_10_000_line_file(path):
    with open(path, 'w', encoding='utf8') as f:
        f.write("x = 1\n")
        for i in range(10_000):
            f.write(f"if (x > {i}) x = {i*2}\n")

def main():
    generate_10_000_line_file("c:/tmp/10_000_lines.lox")

if __name__ == '__main__':
    main()

'''
test:
    let mut i = 0;
    let mut scanner = Scanner::new(file);
    while scanner.scan_token().unwrap().token_type != TokenType::Eof {
        i += 1;
    }

    println!("{i}");

single struct
time ./target/release/rlox.exe 'c:/tmp/10_000_lines.lox'
90003

________________________________________________________
Executed in   34.23 millis    fish           external
   usr time    0.00 millis    0.00 micros    0.00 millis
   sys time   15.00 millis    0.00 micros   15.00 millis

________________________________________________________
Executed in   33.46 millis    fish           external
   usr time    0.00 millis    0.00 micros    0.00 millis
   sys time   15.00 millis    0.00 micros   15.00 millis

Result<Token, ErrorToken>
time ./target/release/rlox.exe 'c:/tmp/10_000_lines.lox'
90003

________________________________________________________
Executed in   35.66 millis    fish           external
   usr time   15.00 millis    0.00 micros   15.00 millis
   sys time    0.00 millis    0.00 micros    0.00 millis

Executed in   33.93 millis    fish           external
   usr time    0.00 millis    0.00 micros    0.00 millis
   sys time   15.00 millis    0.00 micros   15.00 millis


'''
