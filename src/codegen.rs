use std::collections::HashMap;

use crate::parser::Node;

fn traverse_ast_and_alloc_idofs(ast: &Node, map: &mut IdAddrMap) {
    match ast {
        Node::Id(s) => {
            map.get_ofs(s.clone());
        }
        Node::Num(_) => {}
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
    }
}

fn generate_prologue(frame: &StackFrame) {
    println!(".globl main");
    println!("main:");
    println!("addi sp,sp,-{}", frame.size);
    println!("sd ra,{}(sp)", frame.size - 8);
    println!("sd s0,{}(sp)", frame.size - 16);
    println!("addi s0,sp,{}", frame.size);
    println!("sw a0,-20(s0)");
    println!("sd a1,-32(s0)");
}

fn generate_epilogue(frame: &StackFrame) {
    println!("ld a5,0(sp)");
    println!("addi sp,sp,16");
    println!("mv a0,a5");
    println!("ld ra,{}(sp)", frame.size - 8);
    println!("ld s0,{}(sp)", frame.size - 16);
    println!("addi sp,sp,{}", frame.size);
    println!("jr ra");
}

pub fn generate(ast: &Node) {
    let mut frame = StackFrame::new(ast);
    generate_prologue(&frame);
    generate_node(ast, &mut frame.id_addr_map);
    // for return value.(temporary)
    println!("addi sp,sp,-16");
    generate_epilogue(&frame);
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

    // if key not found, assign new region.
    pub fn get_ofs(&mut self, key: String) -> u64 {
        const OFS_UNIT: u64 = 8;

        if let Some(v) = self.map.get(&key) {
            *v
        } else {
            let ret = self.next_ofs;
            self.map.insert(key, ret);
            self.next_ofs += OFS_UNIT;
            ret
        }
    }
}

// Generate stack machine code. The code is generated in preorder of the AST.
fn generate_node(ast: &Node, map: &mut IdAddrMap) {
    match ast {
        Node::Add(l, r) => {
            generate_node(l, map);
            generate_node(r, map);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("add t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Sub(l, r) => {
            generate_node(l, map);
            generate_node(r, map);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("sub t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Mul(l, r) => {
            generate_node(l, map);
            generate_node(r, map);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("mul t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Div(l, r) => {
            generate_node(l, map);
            generate_node(r, map);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("div t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Lt(l, r) => {
            generate_node(l, map);
            generate_node(r, map);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("slt t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Leq(l, r) => {
            generate_node(l, map);
            generate_node(r, map);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("slt t0,t1,t0");
            println!("xori t0,t0,1");
            println!("sd t0,0(sp)");
        }
        Node::Eq(l, r) => {
            generate_node(l, map);
            generate_node(r, map);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("sub t0,t0,t1");
            println!("seqz t0,t0");
            println!("sd t0,0(sp)");
        }
        Node::Neq(l, r) => {
            generate_node(l, map);
            generate_node(r, map);
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
            generate_node(r, map);
            if let Node::Id(s) = l.as_ref() {
                let ofs = map.get_ofs(s.clone());
                println!("ld t0,0(sp)");
                println!("sd t0,-{}(s0)", ofs); // pre-allocated memory region for variables, base=s0.
            // return value is rhs, so we do not change sp.
            } else {
                panic!("Error");
            }
        }
        Node::Id(s) => {
            // accept uninitialized identifier, value is undefined.
            let ofs = map.get_ofs(s.clone());
            println!("addi sp,sp,-16");
            println!("ld t0,-{}(s0)", ofs);
            println!("sd t0,0(sp)");
        }
        Node::Prog(v) => {
            for node in v {
                generate_node(node, map);
                // discard return value (the value is saved unless overwrite.)
                println!("addi sp,sp,16");
            }
        }
    }
}
