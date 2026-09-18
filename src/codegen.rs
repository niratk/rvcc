use std::fmt::Write;

use crate::ir::{self, BinaryOp};

const STACK_ALIGNMENT: usize = 16;
const SAVED_REGISTERS_SIZE: usize = 16;
const LOCAL_SIZE: usize = 8;
const VALUE_STACK_SIZE: usize = 16;

pub fn generate(program: &ir::Program) -> String {
    let symbols: Vec<&str> = program
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect();
    let mut output = String::from(".globl main\n");
    for function in &program.functions {
        FunctionGenerator::new(&mut output, &symbols, function).generate();
    }
    output
}

struct FrameLayout {
    size: usize,
}

impl FrameLayout {
    fn new(local_count: usize) -> Self {
        let required = SAVED_REGISTERS_SIZE + local_count * LOCAL_SIZE;
        Self {
            size: required.div_ceil(STACK_ALIGNMENT) * STACK_ALIGNMENT,
        }
    }

    fn local_offset(&self, local: ir::LocalId) -> usize {
        SAVED_REGISTERS_SIZE + LOCAL_SIZE * (local.0 + 1)
    }
}

struct FunctionGenerator<'a> {
    output: &'a mut String,
    symbols: &'a [&'a str],
    function: &'a ir::Function,
    frame: FrameLayout,
    next_label: usize,
    stack_depth: usize,
}

impl<'a> FunctionGenerator<'a> {
    fn new(output: &'a mut String, symbols: &'a [&'a str], function: &'a ir::Function) -> Self {
        Self {
            output,
            symbols,
            function,
            frame: FrameLayout::new(function.local_count),
            next_label: 0,
            stack_depth: 0,
        }
    }

    fn generate(mut self) {
        writeln!(self.output, "{}:", self.function.name).unwrap();
        writeln!(self.output, "addi sp,sp,-{}", self.frame.size).unwrap();
        writeln!(self.output, "sd ra,{}(sp)", self.frame.size - 8).unwrap();
        writeln!(self.output, "sd s0,{}(sp)", self.frame.size - 16).unwrap();
        writeln!(self.output, "addi s0,sp,{}", self.frame.size).unwrap();

        for (register, local) in self.function.params.iter().enumerate() {
            writeln!(
                self.output,
                "sd a{},-{}(s0)",
                register,
                self.frame.local_offset(*local)
            )
            .unwrap();
        }

        self.generate_block(&self.function.body);
        debug_assert_eq!(self.stack_depth, 0);
        writeln!(self.output, "li a0,0").unwrap();
        writeln!(self.output, ".Lreturn{}:", self.function.id.0).unwrap();
        writeln!(self.output, "ld ra,{}(sp)", self.frame.size - 8).unwrap();
        writeln!(self.output, "ld s0,{}(sp)", self.frame.size - 16).unwrap();
        writeln!(self.output, "addi sp,sp,{}", self.frame.size).unwrap();
        writeln!(self.output, "jr ra").unwrap();
    }

    fn generate_block(&mut self, block: &ir::Block) {
        for statement in &block.statements {
            let depth = self.stack_depth;
            self.generate_stmt(statement);
            debug_assert_eq!(self.stack_depth, depth, "statements must balance the stack");
        }
    }

    fn generate_stmt(&mut self, statement: &ir::Stmt) {
        match statement {
            ir::Stmt::Expr(expr) => {
                self.generate_expr(expr);
                self.discard_value();
            }
            ir::Stmt::Return(expr) => {
                self.generate_expr(expr);
                self.pop_into("a0");
                writeln!(self.output, "j .Lreturn{}", self.function.id.0).unwrap();
            }
            ir::Stmt::Block(block) => self.generate_block(block),
            ir::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let label = self.new_label();
                self.generate_expr(condition);
                self.pop_into("t0");
                if let Some(else_branch) = else_branch {
                    writeln!(
                        self.output,
                        "beqz t0,.Lelse{}_{}",
                        self.function.id.0, label
                    )
                    .unwrap();
                    self.generate_stmt(then_branch);
                    writeln!(self.output, "j .Lend{}_{}", self.function.id.0, label).unwrap();
                    writeln!(self.output, ".Lelse{}_{}:", self.function.id.0, label).unwrap();
                    self.generate_stmt(else_branch);
                } else {
                    writeln!(self.output, "beqz t0,.Lend{}_{}", self.function.id.0, label).unwrap();
                    self.generate_stmt(then_branch);
                }
                writeln!(self.output, ".Lend{}_{}:", self.function.id.0, label).unwrap();
            }
            ir::Stmt::While { condition, body } => {
                let label = self.new_label();
                writeln!(self.output, ".Lbegin{}_{}:", self.function.id.0, label).unwrap();
                self.generate_expr(condition);
                self.pop_into("t0");
                writeln!(self.output, "beqz t0,.Lend{}_{}", self.function.id.0, label).unwrap();
                self.generate_stmt(body);
                writeln!(self.output, "j .Lbegin{}_{}", self.function.id.0, label).unwrap();
                writeln!(self.output, ".Lend{}_{}:", self.function.id.0, label).unwrap();
            }
            ir::Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(init) = init {
                    self.generate_expr(init);
                    self.discard_value();
                }
                let label = self.new_label();
                writeln!(self.output, ".Lbegin{}_{}:", self.function.id.0, label).unwrap();
                if let Some(condition) = condition {
                    self.generate_expr(condition);
                    self.pop_into("t0");
                    writeln!(self.output, "beqz t0,.Lend{}_{}", self.function.id.0, label).unwrap();
                }
                self.generate_stmt(body);
                if let Some(update) = update {
                    self.generate_expr(update);
                    self.discard_value();
                }
                writeln!(self.output, "j .Lbegin{}_{}", self.function.id.0, label).unwrap();
                writeln!(self.output, ".Lend{}_{}:", self.function.id.0, label).unwrap();
            }
            ir::Stmt::Decl { ctype, target } => {
                todo!()
            }
        }
    }

    fn generate_expr(&mut self, expr: &ir::Expr) {
        let depth = self.stack_depth;
        match expr {
            ir::Expr::Number(number) => {
                writeln!(self.output, "li t0,{number}").unwrap();
                self.push_from("t0");
            }
            ir::Expr::Local(local) => {
                writeln!(
                    self.output,
                    "ld t0,-{}(s0)",
                    self.frame.local_offset(*local)
                )
                .unwrap();
                self.push_from("t0");
            }
            ir::Expr::Assign { target, value } => {
                self.generate_expr(value);
                writeln!(self.output, "ld t0,0(sp)").unwrap();
                writeln!(
                    self.output,
                    "sd t0,-{}(s0)",
                    self.frame.local_offset(*target)
                )
                .unwrap();
            }
            ir::Expr::Binary { op, lhs, rhs } => {
                self.generate_expr(lhs);
                self.generate_expr(rhs);
                self.pop_into("t1");
                writeln!(self.output, "ld t0,0(sp)").unwrap();
                match op {
                    BinaryOp::Add => writeln!(self.output, "add t0,t0,t1"),
                    BinaryOp::Sub => writeln!(self.output, "sub t0,t0,t1"),
                    BinaryOp::Mul => writeln!(self.output, "mul t0,t0,t1"),
                    BinaryOp::Div => writeln!(self.output, "div t0,t0,t1"),
                    BinaryOp::Less => writeln!(self.output, "slt t0,t0,t1"),
                    BinaryOp::LessEqual => {
                        writeln!(self.output, "slt t0,t1,t0").unwrap();
                        writeln!(self.output, "xori t0,t0,1")
                    }
                    BinaryOp::Equal => {
                        writeln!(self.output, "sub t0,t0,t1").unwrap();
                        writeln!(self.output, "seqz t0,t0")
                    }
                    BinaryOp::NotEqual => {
                        writeln!(self.output, "sub t0,t0,t1").unwrap();
                        writeln!(self.output, "snez t0,t0")
                    }
                }
                .unwrap();
                writeln!(self.output, "sd t0,0(sp)").unwrap();
            }
            ir::Expr::Call { function, args } => {
                for arg in args {
                    self.generate_expr(arg);
                }
                for register in (0..args.len()).rev() {
                    let register_name = format!("a{register}");
                    self.pop_into(&register_name);
                }
                writeln!(self.output, "call {}", self.symbols[function.0]).unwrap();
                self.push_from("a0");
            }
        }
        debug_assert_eq!(
            self.stack_depth,
            depth + 1,
            "expressions must push exactly one value"
        );
    }

    fn new_label(&mut self) -> usize {
        let label = self.next_label;
        self.next_label += 1;
        label
    }

    fn push_from(&mut self, register: &str) {
        writeln!(self.output, "addi sp,sp,-{VALUE_STACK_SIZE}").unwrap();
        writeln!(self.output, "sd {register},0(sp)").unwrap();
        self.stack_depth += 1;
    }

    fn pop_into(&mut self, register: &str) {
        debug_assert!(self.stack_depth > 0);
        writeln!(self.output, "ld {register},0(sp)").unwrap();
        writeln!(self.output, "addi sp,sp,{VALUE_STACK_SIZE}").unwrap();
        self.stack_depth -= 1;
    }

    fn discard_value(&mut self) {
        debug_assert!(self.stack_depth > 0);
        writeln!(self.output, "addi sp,sp,{VALUE_STACK_SIZE}").unwrap();
        self.stack_depth -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_source(source: &str) -> String {
        let tokens = crate::lexer::lex(source).unwrap();
        let ast = crate::parser::parse(&tokens).unwrap();
        let program = crate::sema::analyze(ast).unwrap();
        generate(&program)
    }

    #[test]
    fn generates_parameter_slots_and_call_registers() {
        let assembly = generate_source(
            "int add(int a,int b) { return a+b; } int main() { return add(20,22); }",
        );

        assert!(assembly.contains("add:\naddi sp,sp,-32\n"));
        assert!(assembly.contains("sd a0,-24(s0)\nsd a1,-32(s0)\n"));
        assert!(assembly.contains("ld a1,0(sp)\naddi sp,sp,16\nld a0,0(sp)"));
        assert!(assembly.contains("call add\naddi sp,sp,-16\nsd a0,0(sp)"));
    }

    #[test]
    fn labels_are_unique_between_functions() {
        let assembly = generate_source(
            "int f(int a) { if (a) return 1; return 0; } int main() { while (0) {} return f(1); }",
        );

        assert!(assembly.contains(".Lend0_0:"));
        assert!(assembly.contains(".Lreturn0:"));
        assert!(assembly.contains(".Lbegin1_0:"));
        assert!(assembly.contains(".Lreturn1:"));
        assert!(assembly.contains("j .Lreturn0"));
        assert!(assembly.contains("j .Lreturn1"));
    }

    #[test]
    fn fallthrough_returns_zero() {
        let assembly = generate_source("int main() {}");
        assert!(assembly.contains("li a0,0\n.Lreturn0:\n"));
    }

    #[test]
    fn nested_calls_preserve_earlier_arguments() {
        let assembly = generate_source(
            "int id(int x) { return x; } int add(int a,int b) { return a+b; } int main() { return add(id(1), id(2)); }",
        );
        assert_eq!(assembly.matches("call id\n").count(), 2);
        assert!(assembly.contains("call add\n"));
    }
}
