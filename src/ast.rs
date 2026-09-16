#[derive(Debug, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Stmt {
    Expr(Expr),
    Return(Expr),
    Block(Block),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    For {
        init: Option<Expr>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Box<Stmt>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Number(u64),
    Variable(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Less,
    LessEqual,
    Equal,
    NotEqual,
}
