# rvcc - C compiler for risc-v

## Prerequisite

### macOS

- [RISC-V tool chain](https://github.com/riscv-software-src/homebrew-riscv)
- [Rust Environment](https://rust-lang.org/ja/tools/install/)

## Grammar

```ebnf
prog   = func* ;
func   = ident, "(", params?, ")", block ;
params = ident, {",", ident} ;
args   = expr, {",", expr} ;
block  = "{", stmt*, "}" ;
stmt   = ("return"?, expr, ";")
         | ("if", "(", expr, ")", stmt, ("else", stmt)?)
         | ("while", "(", expr, ")", stmt)
         | ("for", "(", expr?, ";", expr? ";", expr? ")",stmt)
         | ("{", stmt*, "}") ;
expr   = assign ;
assign = eq ("=", assign)?;
eq     = cmp, (("==" | "!="), cmp)* ;
cmp    = add, ((">" | ">=" | "<" | "<="), add)* ;
add    = mul, { ("+" | "-"), mul } ;
mul    = unary, { ("*" | "/"), unary } ;
unary  = ("+" | "-")?, factor ;
factor = number
        | ident, ("(",args?,")")?
        | "(", expr, ")" ;
number = digit, { digit } ;
digit  = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
ident  = [a-z A-Z], [a-z A-Z 0-9]*;
```

## Example

```c
add(x, y) {
    return x + y;
}

main() {
    return add(3, 10);
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

- [x] return keyword support
    - [x] duplicate implicit return value
- [x] control syntax support
    - [x] if, else, while, for
        - [x] lexer
            - [x] define new grammar
            - [x] mod lexer
        - [x] parser
        - [x] codegen
            - [x] change generator to receive ctx, not frame nor map.
            - [x] impl
    - [x] (defer do...while, goto, continue, break)
- [x] block support
- [x] fn support
    - [x] define the function syntax and AST
        - [x] add `,` to the lexer
        - [x] add `Func { name, params, body }` and `Call { name, args }` nodes
        - [x] change `Prog` from a list of statements to a list of functions
        - [x] parse function definitions with `prog = func*`
        - [x] parse parameter lists with `params = ident, { ",", ident }`
        - [x] parse function calls with `args = expr, { ",", expr }`
    - [x] resolve function names and variable scopes before code generation
        - [x] collect function signatures before code generation to support forward calls and recursion
        - [x] reject duplicate function names, duplicate parameter names, and more than 8 parameters or arguments
        - [x] reject unknown callees and arity mismatches
        - [x] resolve block- and `for`-scoped variables to `LocalId` values
    - [x] make code generation operate on one function at a time
        - [x] create a separate context and stack frame for each function
        - [x] emit unique control-flow and return labels
    - [x] implement parameters and calls with up to 8 arguments
        - [x] copy `a0`-`a7` into parameter slots
        - [x] preserve argument values across nested calls
        - [x] keep `sp` 16-byte aligned at each `call`
    - [x] require an explicit zero-parameter `main`
    - [x] cover forward calls, nested calls, recursion, scopes, and invalid programs end-to-end

## Limitation

- This is an expression compiler rather than a complete C compiler. It only
  supports the grammar described above; declarations, types, pointers, arrays,
  and comments are not supported.
- The entire source program must be passed as a single command-line argument,
  and every statement must end with a semicolon.
- Variables hold 64-bit integer values and are implicitly created by their first
  assignment. Blocks and `for` statements introduce scopes; reading a variable
  before it becomes visible is a compile error.
- Functions accept at most eight integer arguments. The program must define
  `main()` with no parameters. Reaching the end of a function returns `0`.
- Integer overflow and division by zero are not diagnosed.
- Diagnostics do not yet include source locations.
- The generated assembly targets 64-bit RISC-V.
- risc-v immidiate offset (e.g.`12(s0)`) is limited to 12 bits. we ignore this for now (we need to calculate address with register for larger frames, but we ignore this for now.)

## Reference

- [低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)
