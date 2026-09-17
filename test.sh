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

assert_error() {
  input="$1"

  if cargo run -- "$input" > /dev/null 2>&1; then
    echo "expected compilation to fail: $input"
    exit 1
  fi
  echo "$input => compile error"
}

assert 0 "main() { return 3 - 3; }"
assert 12 "main() { int a; a=12; return a; }"
assert 50 "main() { int a; int b; int c; a = 23; b = 3 + 5 * 2; c = b + (a + b > a); return a + b + c; }"
assert 23 "main() { int a; int b; int c; a = 23; return 23; b = 3 + 5 * 2; c = b + (a + b > a); return a + b + c; }"
assert 3 "main() { int a; a = 0; if (1) a = 3; return a; }"
assert 0 "main() { int a; a = 0; if (0) a = 3; return a; }"
assert 3 "main() { if (1) return 3; else return 5; }"
assert 5 "main() { if (0) return 3; else return 5; }"
assert 10 "main() { int i; i = 0; while (i < 10) i = i + 1; return i; }"
assert 55 "main() { int sum; int i; sum = 0; for (i = 0; i <= 10; i = i + 1) sum = sum + i; return sum; }"
assert 7 "main() { for (;;) return 7; }"
assert 3 "main() { int a; a = 0; if (1) { a = 1; a = a + 2; } return a; }"
assert 10 "main() { int i; int sum; i = 0; sum = 0; while (i < 4) { i = i + 1; sum = sum + i; } return sum; }"
assert 6 "main() { int sum; int i; sum = 0; for (i = 0; i < 4; i = i + 1) { sum = sum + i; } return sum; }"
assert 2 "main() { if (1) { if (0) { return 1; } else { return 2; } } return 3; }"
assert 0 "main() { int value; value = 42; }"
assert 42 "answer() { return 42; } main() { return answer(); }"
assert 42 "main() { return identity(42); } identity(value) { return value; }"
assert 36 "sum8(a,b,c,d,e,f,g,h) { return a+b+c+d+e+f+g+h; } main() { return sum8(1,2,3,4,5,6,7,8); }"
assert 21 "add(a,b) { return a+b; } identity(value) { return value; } main() { return add(identity(10), identity(11)); }"
assert 120 "factorial(n) { if (n <= 1) return 1; return n * factorial(n - 1); } main() { return factorial(5); }"
assert 7 "choose(value) { if (value) return 7; return 9; } main() { return choose(1); }"

assert_error "main() { return missing; }"
assert_error "main() { { int i; for (i = 0; i < 1; i = i + 1) {} } return i; }"
assert_error "main() { return f(); } f(value) { return value; }"
assert_error "main() {} main() {}"
assert_error "main(value) { return value; }"
assert_error "main() { return nine(1,2,3,4,5,6,7,8,9); } nine(a,b,c,d,e,f,g,h,i) { return 0; }"

echo OK
