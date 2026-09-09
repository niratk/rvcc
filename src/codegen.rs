use std::collections::HashMap;

use crate::parser::Node;

pub fn generate(ast: &Node) {
    let mut map = IdAddrMap::new();
    generate_node(ast, &mut map);
    // for return value.(temporary)
    println!("addi sp,sp,-16");
}

struct IdAddrMap {
    next_ofs: u64,
    map: HashMap<String, u64>,
}

impl IdAddrMap {
    pub fn new() -> Self {
        Self {
            // if stack grows too long so that reaches this beginning ofs, this will break. We should improve this.
            next_ofs: 1600,
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
