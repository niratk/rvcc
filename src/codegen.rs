use crate::parser::Node;

// Generate stack machine code. The code is generated in preorder of the AST.
pub fn generate(ast: &Node) {
    match ast {
        Node::Add(l, r) => {
            generate(l);
            generate(r);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("add t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Sub(l, r) => {
            generate(l);
            generate(r);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("sub t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Mul(l, r) => {
            generate(l);
            generate(r);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("mul t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Div(l, r) => {
            generate(l);
            generate(r);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("div t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Lt(l, r) => {
            generate(l);
            generate(r);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("slt t0,t0,t1");
            println!("sd t0,0(sp)");
        }
        Node::Leq(l, r) => {
            generate(l);
            generate(r);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("slt t0,t1,t0");
            println!("xori t0,t0,1");
            println!("sd t0,0(sp)");
        }
        Node::Eq(l, r) => {
            generate(l);
            generate(r);
            println!("ld t1,0(sp)");
            println!("addi sp,sp,16");
            println!("ld t0,0(sp)");
            println!("sub t0,t0,t1");
            println!("seqz t0,t0");
            println!("sd t0,0(sp)");
        }
        Node::Neq(l, r) => {
            generate(l);
            generate(r);
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
    }
}
