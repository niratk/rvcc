# rvcc - C compiler for risc-v

## Prerequisite

### macOS

- [RISC-V tool chain](https://github.com/riscv-software-src/homebrew-riscv)
- [Rust Environment](https://rust-lang.org/ja/tools/install/)

## Grammar

```ebnf
prog   = stmt* ;
stmt   = "return"?, expr, ";" ;
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

## Example

```c
x = 3;
y = 5;
return x + y * 2;
```

This program returns `13` from `main`.

## Stack Frame Explained

after prologue

```
高アドレス
元のsp → +----------------+
          | 呼び出し元     |
s0     → +----------------+
          | 保存ra  8byte | -8(s0)
          | 保存s0  8byte | -16(s0)
          | argc    4byte | -20(s0)
          | 空き    4byte |
sp     → | argv    8byte | -32(s0)
          +----------------+
低アドレス
```

after allocating space for scope variables (planned)

```
高アドレス
元のsp → +----------------+ ← s0
          | 呼び出し元     |
          +----------------+
          | 保存ra  8byte | -8(s0)
          | 保存s0  8byte | -16(s0)
          | argc    4byte | -20(s0)
          | 空き    4byte |
          | argv    8byte | -32(s0)
          +----------------+
          | var[0]  8byte |
          | var[1]  8byte |
          |      ...       |
          | var[n-1] 8byte|
sp     → +----------------+
低アドレス
```

## Todo

- [x] return keyword support
    - [x] duplicate implicit return value
- [ ] control syntax support
    - [ ] if, else, while, for
    - [ ] (defer do...while, goto, continue, break)
- [ ] block support
- [ ] fn support

## Limitation

- This is an expression compiler rather than a complete C compiler. It only
  supports the grammar described above; declarations, types, functions,
  control flow, pointers, arrays, and comments are not supported.
- The entire source program must be passed as a single command-line argument,
  and every statement must end with a semicolon.
- Variables are implicitly created on first use, have no scope, and hold
  64-bit integer values. Reading a variable before assigning to it produces an
  undefined value.
- Integer overflow and division by zero are not diagnosed.
- Invalid input is reported by a panic rather than a user-friendly compiler
  diagnostic.
- The generated assembly targets 64-bit RISC-V and currently emits only a
  `main` function whose return value is the value of the final statement.
- risc-v immidiate offset (e.g.`12(s0)`) is limited to 12 bits. we ignore this for now (we need to calculate address with register for larger frames, but we ignore this for now.)

## Reference

- [低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)
