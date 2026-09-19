# rvcc - C compiler for risc-v

## Prerequisite

### macOS

- [RISC-V tool chain](https://github.com/riscv-software-src/homebrew-riscv)
- [Rust Environment](https://rust-lang.org/ja/tools/install/)

## Grammar

```ebnf
prog   = func* ;
func   = ident, ident, "(", params?, ")", block ;
params = ident, ident, {",", ident, ident} ;
args   = expr, {",", expr} ;
block  = "{", stmt*, "}" ;
stmt   = ("return"?, expr, ";")
         | ("if", "(", expr, ")", stmt, ("else", stmt)?)
         | ("while", "(", expr, ")", stmt)
         | ("for", "(", expr?, ";", expr? ";", expr? ")",stmt)
         | ("{", stmt*, "}")
         | (ident, ident, ";") ; // variable declare
expr   = assign ;
assign = eq ("=", assign)?;
eq     = cmp, (("==" | "!="), cmp)* ;
cmp    = add, ((">" | ">=" | "<" | "<="), add)* ;
add    = mul, { ("+" | "-"), mul } ;
mul    = unary, { ("*" | "/"), unary } ;
unary  = ("+" | "-")?, factor
        | "*", unary
        | "&", unary ;
factor = number
        | ident, ("(",args?,")")?
        | "(", expr, ")" ;
number = digit, { digit } ;
digit  = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
ident  = [a-z A-Z], [a-z A-Z 0-9]*;
```

## Example

```c
int add(int x, int y) {
    return x + y;
}

int main() {
    int a;
    int b;
    a = 3;
    b = 10;
    return add(a, b);
}
```

This program returns `13` from `main`.

## Compiler Pipeline

1. The parser builds a syntax AST containing source-level names.
2. Semantic analysis resolves function names and scoped variables to stable
   `FunctionId` and `LocalId` values.
3. Code generation consumes only the resolved IR and emits one stack frame per
   function.

## Stack Frame

After a function prologue, `s0` points to the caller's stack pointer. Parameters
are copied from `a0`-`a7` into their assigned local slots.

```
高アドレス
元のsp → +----------------+ ← s0
          | 呼び出し元     |
          +----------------+
          | 保存ra  8byte | -8(s0)
          | 保存s0  8byte | -16(s0)
          | local[0] 8byte| -24(s0)
          | local[1] 8byte| -32(s0)
          |      ...       |
          | alignment pad  |
sp     → +----------------+  16-byte aligned
低アドレス
```

## Todo

- [ ] Step 16 unary ops `&`(addr), `*`(deref) support
    - [x] lexer
        - [x] add `&` token
    - [x] parser
        - [x] modify `parse_unary` to accept new grammar.
    - [x] ast
        - [x] define `Expr::Addr(Box<Expr>)` and `Expr::Deref(Box<Expr>)` // accept gracefully in AST, reject in semantic analysis.
    - [x] sema
        - [x] resolve var_name to localid for expr::addr
    - [x] ir
        - [x] define `Ir::Addr(LocalID)`, `Ir::deref(Box<ir::expr>)`
    - [x] codegen
- [ ] current ABI is ILP64. Change this to LP64.

## Limitation

- Variable declarations are limited to one uninitialized variable at a time:
  `int a;` is supported, while `int a,b;` and `int a = 3;` are not. A variable
  must be declared before use; assignment does not implicitly declare it.
- `int` is currently the only type and is represented as a 64-bit value. The IR
  therefore holds only partial type information.
- `&` accepts only a declared variable and returns the address of its local
  stack slot as a 64-bit integer. `*` accepts any expression, treats its result
  as an address, and loads one 64-bit integer from that address.
- Unary `+` is currently discarded while building the AST, so `&+a` and
  `&(+a)` are incorrectly accepted as if they were `&a`.
- Pointer types and pointer validity are not tracked. Consequently, invalid or
  unaligned addresses produced by `*` are not diagnosed by the compiler and may
  fail at runtime.
- This is an expression compiler rather than a complete C compiler. It only
  supports the grammar described above; arrays, declaration initializers,
  comma-separated declarations, and comments are not supported.
- The entire source program must be passed as a single command-line argument,
  and every statement must end with a semicolon.
- Variables hold 64-bit integer values. Blocks and `for` statements introduce
  scopes; using a variable before its declaration or outside its scope is a
  compile error.
- Functions accept at most eight integer arguments. The program must define
  `main()` with no parameters. Reaching the end of a function returns `0`.
- Integer overflow and division by zero are not diagnosed.
- Diagnostics do not yet include source locations.
- The generated assembly targets 64-bit RISC-V.
- risc-v immidiate offset (e.g.`12(s0)`) is limited to 12 bits. we ignore this for now (we need to calculate address with register for larger frames, but we ignore this for now.)

## Reference

- [低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)
  This project has implemented until the Step 17.
