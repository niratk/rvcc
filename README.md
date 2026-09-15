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
- [ ] fn support
    - [ ] define the function syntax and AST
        - [x] add `,` to the lexer
        - [ ] add `Func { name, params, body }` and `Call { name, args }` nodes
        - [ ] change `Prog` from a list of statements to a list of functions
        - [ ] parse function definitions with `prog = func*`
        - [ ] parse parameter lists with `params = ident, { ",", ident }`
        - [ ] parse function calls with `args = expr, { ",", expr }`
    - [ ] resolve function names and variable scopes before code generation
        - [ ] collect function signatures before code generation to support forward calls and recursion
        - [ ] reject duplicate function names, duplicate parameter names, and more than 8 parameters or arguments
        - [ ] reject unknown callees and arity mismatches
        - [ ] maintain a scope stack while allocating and resolving identifiers
        - [ ] start a function scope containing its parameters
        - [ ] push and pop a scope for each block
        - [ ] define an implicitly created variable at its first assignment and keep it visible until the end of that block
        - [ ] resolve reads and assignments from the innermost scope outward
        - [ ] reject reads of names that are not visible at that point
        - [ ] assign distinct stack slots to same-named variables in separate functions or sibling blocks
    - [ ] make code generation operate on one function at a time
        - [ ] create a separate `CodegenContext` and `StackFrame` for each function
        - [ ] emit the function name as its assembly label
        - [ ] generalize prologue and epilogue generation for any function
        - [ ] give each function its own return label and make every `return` jump to it
        - [ ] keep generated control-flow labels unique across the program
    - [ ] implement parameters according to the RISC-V calling convention
        - [ ] reserve stack slots for parameters in the callee's frame
        - [ ] copy `a0`-`a7` into those slots in the prologue
        - [ ] resolve parameter references in the same way as local variables
    - [ ] implement calls with up to 8 arguments
        - [ ] evaluate arguments without overwriting earlier argument values
        - [ ] load argument values into `a0`-`a7` in source order
        - [ ] keep `sp` 16-byte aligned at each `call`
        - [ ] emit `call <name>` and push the return value from `a0` as the value of the call expression
        - [ ] verify nested calls and calls used inside larger expressions
    - [ ] switch from the implicit `main` wrapper to explicit functions
        - [ ] require the input program to define `main`
        - [ ] remove generation of the implicit `main` function
        - [ ] update existing examples and tests to wrap statements in `main() { ... }`
    - [ ] add end-to-end coverage
        - [ ] functions with zero, one, and eight parameters
        - [ ] calls to functions defined before and after the caller
        - [ ] multiple calls, nested calls, recursion, and early returns
        - [ ] variables with the same name in different functions and sibling blocks
        - [ ] invalid definitions, invalid arity, and calls with more than 8 arguments

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
