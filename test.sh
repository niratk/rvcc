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

assert 0 "int main() { return 3 - 3; }"
assert 12 "int main() { int a; a=12; return a; }"
assert 50 "int main() { int a; int b; int c; a = 23; b = 3 + 5 * 2; c = b + (a + b > a); return a + b + c; }"
assert 23 "int main() { int a; int b; int c; a = 23; return 23; b = 3 + 5 * 2; c = b + (a + b > a); return a + b + c; }"
assert 3 "int main() { int a; a = 0; if (1) a = 3; return a; }"
assert 0 "int main() { int a; a = 0; if (0) a = 3; return a; }"
assert 3 "int main() { if (1) return 3; else return 5; }"
assert 5 "int main() { if (0) return 3; else return 5; }"
assert 10 "int main() { int i; i = 0; while (i < 10) i = i + 1; return i; }"
assert 55 "int main() { int sum; int i; sum = 0; for (i = 0; i <= 10; i = i + 1) sum = sum + i; return sum; }"
assert 7 "int main() { for (;;) return 7; }"
assert 3 "int main() { int a; a = 0; if (1) { a = 1; a = a + 2; } return a; }"
assert 10 "int main() { int i; int sum; i = 0; sum = 0; while (i < 4) { i = i + 1; sum = sum + i; } return sum; }"
assert 6 "int main() { int sum; int i; sum = 0; for (i = 0; i < 4; i = i + 1) { sum = sum + i; } return sum; }"
assert 2 "int main() { if (1) { if (0) { return 1; } else { return 2; } } return 3; }"
assert 0 "int main() { int value; value = 42; }"
assert 42 "int answer() { return 42; } int main() { return answer(); }"
assert 42 "int main() { return identity(42); } int identity(int value) { return value; }"
assert 36 "int sum8(int a,int b,int c,int d,int e,int f,int g,int h) { return a+b+c+d+e+f+g+h; } int main() { return sum8(1,2,3,4,5,6,7,8); }"
assert 21 "int add(int a,int b) { return a+b; } int identity(int value) { return value; } int main() { return add(identity(10), identity(11)); }"
assert 120 "int factorial(int n) { if (n <= 1) return 1; return n * factorial(n - 1); } int main() { return factorial(5); }"
assert 7 "int choose(int value) { if (value) return 7; return 9; } int main() { return choose(1); }"
assert 10 "int main(){int a; int b; b = &a; a = 10; return *b;}"
assert 42 "int main() { int value; value = 42; return *&value; }"
assert 73 "int main() { int value; int pointer; int pointerpointer; value = 73; pointer = &value; pointerpointer = &pointer; return **pointerpointer; }"
assert 42 "int identity(int value) { return value; } int main() { int value; value = 39; return *identity(&value) + 3; }"
assert 51 "int read(int value) { int pointer; pointer = &value; return *pointer; } int main() { return read(51); }"

assert_error "int main() { return missing; }"
assert_error "int main() { { int i; for (i = 0; i < 1; i = i + 1) {} } return i; }"
assert_error "int main() { return f(); } int f(int value) { return value; }"
assert_error "int main() {} int main() {}"
assert_error "int main(int value) { return value; }"
assert_error "int main() { return nine(1,2,3,4,5,6,7,8,9); } int nine(int a,int b,int c,int d,int e,int f,int g,int h,int i) { return 0; }"
assert_error "int main() { return &1; }"
assert_error "int main() { int value; return &(value + 1); }"
assert_error "int main() { return &helper(); } int helper() { return 1; }"
assert_error "int main() { int value; return &*value; }"

echo OK
