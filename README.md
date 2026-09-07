# rvcc - C compiler for risc-v

## Prerequisite

### macOS

- [RISC-V tool chain](https://github.com/riscv-software-src/homebrew-riscv)
- [Rust Environment](https://rust-lang.org/ja/tools/install/)

## Grammar

```ebnf
expr   = term, { ("+" | "-"), term } ;
term   = factor, { ("*" | "/"), factor } ;
factor = number | "(", expr, ")" ;
number = digit, { digit } ;
digit  = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
```

空白文字はトークン間で無視されます。`*` と `/` は `+` と `-` より優先され、同じ優先順位の演算子は左結合です。現在、単項の `+` と `-` は扱いません。

## Reference

- [低レイヤを知りたい人のためのCコンパイラ作成入門](https://www.sigbus.info/compilerbook)
