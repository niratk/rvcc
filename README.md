# rvcc - C compiler for risc-v

## Prerequisite

### macOS

- [RISC-V tool chain](https://github.com/riscv-software-src/homebrew-riscv)
- [Rust Environment](https://rust-lang.org/ja/tools/install/)

## Grammar

```ebnf
prog   = stmt* ;
stmt   = expr, ";" ;
expr   = assign ;
assign = eq ("=", assign)?;
eq     = cmp, (("==" | "!="), cmp)* ;
cmp    = add, ((">" | ">=" | "<" | "<="), add)* ;
add    = mul, { ("+" | "-"), mul } ;
mul    = unary, { ("*" | "/"), unary } ;
unary  = ("+" | "-")?, factor ;
factor = number | ident | "(", expr, ")" ;
number = digit, { digit } ;
digit  = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
ident  = [a-z A-Z], [a-z A-Z 0-9]*;
```

## Limitation

- This is an expression compiler rather than a complete C compiler. It only
  supports the grammar described above; declarations, types, functions,
  control flow, pointers, arrays, and comments are not supported.
- The entire source program must be passed as a single command-line argument,
  and every statement must end with a semicolon.
- Variables are implicitly created on first use, have no scope, and hold
  64-bit integer values. Reading a variable before assigning to it produces an
  undefined value.
- Variable storage uses fixed offsets from the stack frame without reserving
  the corresponding stack space. Programs with many variables or deeply
  nested expressions may corrupt the stack, and sufficiently large variable
  offsets cannot be encoded by the generated instructions.
- Integer overflow and division by zero are not diagnosed.
- Invalid input is reported by a panic rather than a user-friendly compiler
  diagnostic.
- The generated assembly targets 64-bit RISC-V and currently emits only a
  `main` function whose return value is the value of the final statement.

## Reference

- [低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)
