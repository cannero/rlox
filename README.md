# Bytecode virtual machine for Lox in rust

Implement https://www.craftinginterpreters.com/a-bytecode-virtual-machine.html in rust.
But simplified by no string interning or a constants cache.

Progress: functions

## Usage
3 different ways:

- Compile and run:
```fish
# with debug switch
cargo run -- --debug 'c:/tmp/function.lox'
```

- Only compile, will create *.loxer file:
```fish
cargo run -- --compile --debug 'c:/tmp/function.lox'
```

- Run compiled `loxer` program:
```fish
cargo run -- --run --debug 'c:/tmp/function.loxer' 
```

## other impl
- https://github.com/LevitatingBusinessMan/loxidation
- https://github.com/ryotsu/rox
- https://github.com/aurelilia/cloxrs
- https://github.com/abesto/clox-rs
- https://github.com/ajeetdsouza/loxcraft
- https://github.com/adambiltcliffe/rlox

