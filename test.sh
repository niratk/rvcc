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

assert 0 0
assert 42 42
assert 3 "2+143-141-1"
assert 8 "(1+3)*(8/(9-5))"
assert 12 "-3+12/-6+15+4/2-9+9"
assert 18 "+3+12/-6+15+4/2-9+9"

echo OK
