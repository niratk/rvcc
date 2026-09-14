#!/bin/bash
tmp_dir="$(mktemp -d)" || exit 1
trap 'rm -rf "$tmp_dir"' EXIT

assert() {
  expected="$1"
  input="$2"

  if ! cargo run -- "$input" > "$tmp_dir/tmp.s"; then
    echo "failed to compile source: $input"
    exit 1
  fi
  if ! riscv64-unknown-elf-gcc -o "$tmp_dir/tmp" "$tmp_dir/tmp.s"; then
    echo "failed to assemble source: $input"
    exit 1
  fi
  spike pk "$tmp_dir/tmp"
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 0 "return 3 - 3;"
assert 12 "a=12;return a;"
assert 50 "a = 23; b = 3 + 5 * 2; c = b + (a + b > a); return a + b + c;"
assert 23 "a = 23; return 23; b = 3 + 5 * 2; c = b + (a + b > a); return a + b + c;"
assert 3 "a = 0; if (1) a = 3; return a;"
assert 0 "a = 0; if (0) a = 3; return a;"
assert 3 "if (1) return 3; else return 5;"
assert 5 "if (0) return 3; else return 5;"
assert 10 "i = 0; while (i < 10) i = i + 1; return i;"
assert 55 "sum = 0; for (i = 0; i <= 10; i = i + 1) sum = sum + i; return sum;"
assert 7 "for (;;) return 7;"
assert 3 "a = 0; if (1) { a = 1; a = a + 2; } return a;"
assert 10 "i = 0; sum = 0; while (i < 4) { i = i + 1; sum = sum + i; } return sum;"
assert 6 "sum = 0; for (i = 0; i < 4; i = i + 1) { sum = sum + i; } return sum;"
assert 2 "if (1) { if (0) { return 1; } else { return 2; } } return 3;"

echo OK
