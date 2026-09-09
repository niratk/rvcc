#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  cargo run -- "$input" > tmp.s
  riscv64-unknown-elf-gcc -o tmp tmp.s
  spike pk ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 0 "3 - 3;"
assert 12 "a=12;a;"
assert 50 "a = 23; b = 3 + 5 * 2; c = b + (a + b > a); a + b + c;"

echo OK
