use std::collections::HashMap;

use crate::parser::Node;

#[cfg(test)]
thread_local! {
    static GENERATED_ASSEMBLY: std::cell::RefCell<String> = const {
        std::cell::RefCell::new(String::new())
    };
}

#[cfg(test)]
macro_rules! println {
    ($($arg:tt)*) => {{
        use std::fmt::Write;
        GENERATED_ASSEMBLY.with(|assembly| {
            writeln!(assembly.borrow_mut(), $($arg)*).unwrap();
        });
    }};
}

fn traverse_ast_and_alloc_idofs(ast: &Node, map: &mut IdAddrMap) {
    match ast {
        Node::Id(s) => {
            map.allocate(s);
        }
        Node::Num(_) => {}
        Node::Return(e) => {
            traverse_ast_and_alloc_idofs(e, map);
        }
        Node::Add(l, r)
        | Node::Sub(l, r)
        | Node::Mul(l, r)
        | Node::Div(l, r)
        | Node::Leq(l, r)
        | Node::Lt(l, r)
        | Node::Eq(l, r)
        | Node::Neq(l, r)
        | Node::Assign(l, r) => {
            traverse_ast_and_alloc_idofs(l, map);
            traverse_ast_and_alloc_idofs(r, map);
        }
        Node::Prog(nodes) => {
            for node in nodes {
                traverse_ast_and_alloc_idofs(node, map);
            }
        }
        Node::If(cond, stmt) | Node::While(cond, stmt) => {
            traverse_ast_and_alloc_idofs(cond, map);
            traverse_ast_and_alloc_idofs(stmt, map);
        }
        Node::IfElse(cond, then_stmt, else_stmt) => {
            traverse_ast_and_alloc_idofs(cond, map);
            traverse_ast_and_alloc_idofs(then_stmt, map);
            traverse_ast_and_alloc_idofs(else_stmt, map);
        }
        Node::For(init, cond, update, stmt) => {
            if let Some(expr) = init {
                traverse_ast_and_alloc_idofs(expr, map);
            }
            if let Some(expr) = cond {
                traverse_ast_and_alloc_idofs(expr, map);
            }
            if let Some(expr) = update {
                traverse_ast_and_alloc_idofs(expr, map);
            }
            traverse_ast_and_alloc_idofs(stmt, map);
        }
    }
}

fn generate_prologue(ctx: &CodegenContext) {
    println!(".globl main");
    println!("main:");
    println!("addi sp,sp,-{}", ctx.frame.size);
    println!("sd ra,{}(sp)", ctx.frame.size - 8);
    println!("sd s0,{}(sp)", ctx.frame.size - 16);
    println!("addi s0,sp,{}", ctx.frame.size);
    println!("sw a0,-20(s0)");
    println!("sd a1,-32(s0)");
}

fn generate_epilogue(ctx: &CodegenContext) {
    println!("ld ra,{}(sp)", ctx.frame.size - 8);
    println!("ld s0,{}(sp)", ctx.frame.size - 16);
    println!("addi sp,sp,{}", ctx.frame.size);
    println!("jr ra");
}

pub fn generate(ast: &Node) {
    let mut ctx = CodegenContext::new(ast);
    generate_prologue(&ctx);
    generate_node(ast, &mut ctx);
    println!(".Lreturn:");
    generate_epilogue(&ctx);
}

struct CodegenContext {
    frame: StackFrame,
    next_label_id: u64,
}

impl CodegenContext {
    fn new(ast: &Node) -> Self {
        Self {
            frame: StackFrame::new(ast),
            next_label_id: 0,
        }
    }
    fn get_new_label(&mut self) -> u64 {
        let label_id = self.next_label_id;
        self.next_label_id += 1;
        label_id
    }
}

struct StackFrame {
    id_addr_map: IdAddrMap,
    size: u64,
}

impl StackFrame {
    fn new(ast: &Node) -> Self {
        const BASE_FRAME_SIZE: u64 = 32;
        const STACK_ALIGNMENT: u64 = 16;

        let mut id_addr_map = IdAddrMap::new();
        traverse_ast_and_alloc_idofs(ast, &mut id_addr_map);
        let required_size = id_addr_map.next_ofs.saturating_sub(8).max(BASE_FRAME_SIZE);
        let size = required_size.div_ceil(STACK_ALIGNMENT) * STACK_ALIGNMENT;

        Self { id_addr_map, size }
    }
}

struct IdAddrMap {
    next_ofs: u64,
    map: HashMap<String, u64>,
}

impl IdAddrMap {
    pub fn new() -> Self {
        Self {
            // the first variable is at -40(s0)
            next_ofs: 40,
            map: HashMap::new(),
        }
    }

    fn allocate(&mut self, key: &str) {
        const OFS_UNIT: u64 = 8;

        if !self.map.contains_key(key) {
            self.map.insert(key.to_string(), self.next_ofs);
            self.next_ofs += OFS_UNIT;
        }
    }

    pub fn get_ofs(&self, key: &str) -> Option<u64> {
        self.map.get(key).copied()
    }
}

// Generate stack machine code. The code is generated in preorder of the AST.
fn generate_node(ast: &Node, ctx: &mut CodegenContext) {
    match ast {
        Node::Add(l, r) => {
            generate_node(l, ctx);
            generate_node(r, ctx);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("add t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Sub(l, r) => {
            generate_node(l, ctx);
            generate_node(r, ctx);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("sub t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Mul(l, r) => {
            generate_node(l, ctx);
            generate_node(r, ctx);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("mul t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Div(l, r) => {
            generate_node(l, ctx);
            generate_node(r, ctx);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("div t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Lt(l, r) => {
            generate_node(l, ctx);
            generate_node(r, ctx);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("slt t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Leq(l, r) => {
            generate_node(l, ctx);
            generate_node(r, ctx);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("slt t0,t1,t0");
            println!("xori t0,t0,1");
            println!("sd t0,0(sp)");
        }
        Node::Eq(l, r) => {
            generate_node(l, ctx);
            generate_node(r, ctx);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("sub t0,t0,t1");
            println!("seqz t0,t0");
            println!("sd t0,0(sp)");
        }
        Node::Neq(l, r) => {
            generate_node(l, ctx);
            generate_node(r, ctx);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("sub t0,t0,t1");
            println!("snez t0,t0");
            println!("sd t0,0(sp)");
        }
        Node::Num(n) => {
            println!("addi sp,sp,-16");
            println!("li t0,{}", n);
            println!("sd t0,0(sp)");
        }
        Node::Assign(l, r) => {
            generate_node(r, ctx);
            if let Node::Id(s) = l.as_ref() {
                let ofs = ctx
                    .frame
                    .id_addr_map
                    .get_ofs(s)
                    .expect("identifier must be allocated before code generation");
                println!("ld t0,0(sp)");
                println!("sd t0,-{}(s0)", ofs); // pre-allocated memory region for variables, base=s0.
            // return value is rhs, so we do not change sp.
            } else {
                panic!("Error");
            }
        }
        Node::Return(e) => {
            generate_node(e, ctx);
            println!("ld a0,0(sp)");
            println!("addi sp,sp,16");
            println!("j .Lreturn");
        }
        Node::Id(s) => {
            // accept uninitialized identifier, value is undefined.
            let ofs = ctx
                .frame
                .id_addr_map
                .get_ofs(s)
                .expect("identifier must be allocated before code generation");
            println!("addi sp,sp,-16");
            println!("ld t0,-{}(s0)", ofs);
            println!("sd t0,0(sp)");
        }
        Node::Prog(v) => {
            for node in v {
                generate_node(node, ctx);
                if matches!(node, Node::Return(_)) {
                    break;
                } else {
                    // discard calc result at the top of the stack (the value is saved unless overwrite.)
                    println!("addi sp,sp,16");
                }
            }
        }
        Node::If(cond, stmt) => {
            let label_id = ctx.get_new_label();
            generate_node(cond, ctx);
            println!("ld t0,0(sp)");
            println!("addi sp,sp,16");
            println!("beqz t0,.Lendif{}", label_id);
            generate_node(stmt, ctx);
            println!("addi sp,sp,16");
            println!(".Lendif{}:", label_id);
            println!("addi sp,sp,-16");
            println!("sd zero,0(sp)");
        }
        Node::IfElse(cond, stmt1, stmt2) => {
            let label_id_else = ctx.get_new_label();
            let label_id_end = ctx.get_new_label();
            generate_node(cond, ctx);
            println!("ld t0,0(sp)");
            println!("addi sp,sp,16");
            println!("beqz t0,.Lelse{}", label_id_else);
            generate_node(stmt1, ctx);
            println!("addi sp,sp,16");
            println!("j .Lendif{}", label_id_end);
            println!(".Lelse{}:", label_id_else);
            generate_node(stmt2, ctx);
            println!("addi sp,sp,16");
            println!(".Lendif{}:", label_id_end);
            println!("addi sp,sp,-16");
            println!("sd zero,0(sp)");
        }
        Node::While(cond, stmt) => {
            let label_id_begin = ctx.get_new_label();
            let label_id_end = ctx.get_new_label();
            println!(".Lbegin{}:", label_id_begin);
            generate_node(cond, ctx);
            println!("ld t0,0(sp)");
            println!("addi sp,sp,16");
            println!("beqz t0,.Lend{}", label_id_end);
            generate_node(stmt, ctx);
            println!("addi sp,sp,16");
            println!("j .Lbegin{}", label_id_begin);
            println!(".Lend{}:", label_id_end);
            println!("addi sp,sp,-16");
            println!("sd zero,0(sp)");
        }
        Node::For(init, cond, cont, stmt) => {
            let label_id_begin = ctx.get_new_label();
            let label_id_end = ctx.get_new_label();
            if let Some(init) = init {
                generate_node(init, ctx);
                println!("addi sp,sp,16");
            }
            println!(".Lbegin{}:", label_id_begin);
            if let Some(cond) = cond {
                generate_node(cond, ctx);
                println!("ld t0,0(sp)");
                println!("addi sp,sp,16");
                println!("beqz t0,.Lend{}", label_id_end);
            }
            generate_node(stmt, ctx);
            println!("addi sp,sp,16");
            if let Some(cont) = cont {
                generate_node(cont, ctx);
                println!("addi sp,sp,16");
            }
            println!("j .Lbegin{}", label_id_begin);
            println!(".Lend{}:", label_id_end);
            println!("addi sp,sp,-16");
            println!("sd zero,0(sp)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_source(source: &str) -> String {
        let tokens = crate::lexer::lex(source).unwrap();
        let ast = crate::parser::parse(&tokens).unwrap();

        GENERATED_ASSEMBLY.with(|assembly| assembly.borrow_mut().clear());
        generate(&ast);
        GENERATED_ASSEMBLY.with(|assembly| std::mem::take(&mut *assembly.borrow_mut()))
    }

    #[test]
    fn generate_if_and_if_else() {
        let assembly = generate_source("if (1) a = 2; if (0) a = 3; else a = 4; return a;");

        assert!(assembly.contains("beqz t0,.Lendif0\n"));
        assert!(assembly.contains("beqz t0,.Lelse1\n"));
        assert!(assembly.contains("j .Lendif2\n"));
        assert!(assembly.contains(".Lelse1:\n"));
        assert!(assembly.contains(".Lendif2:\n"));
    }

    #[test]
    fn generate_while_and_for() {
        let assembly = generate_source(
            "i = 0; while (i < 3) i = i + 1; for (j = 0; j < 2; j = j + 1) i = i + j; return i;",
        );

        assert!(assembly.contains(".Lbegin0:\n"));
        assert!(assembly.contains("beqz t0,.Lend1\n"));
        assert!(assembly.contains("j .Lbegin0\n"));
        assert!(assembly.contains(".Lbegin2:\n"));
        assert!(assembly.contains("beqz t0,.Lend3\n"));
        assert!(assembly.contains("j .Lbegin2\n"));
    }

    #[test]
    fn generate_return_from_control_statement() {
        let assembly = generate_source("for (;;) return 7;");

        assert!(assembly.contains("ld a0,0(sp)\naddi sp,sp,16\nj .Lreturn\n"));
        assert!(assembly.contains(".Lreturn:\n"));
    }
}
