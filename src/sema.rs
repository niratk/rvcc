use std::{collections::HashMap, fmt};

use crate::{
    ast,
    ir::{self, CType},
};

const MAX_ARGUMENTS: usize = 8;

#[derive(Debug, PartialEq, Eq)]
pub struct SemanticError(String);

impl SemanticError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy)]
struct Signature {
    id: ir::FunctionId,
    arity: usize,
}

pub fn analyze(program: ast::Program) -> Result<ir::Program, SemanticError> {
    let mut signatures = HashMap::new();
    for (index, function) in program.functions.iter().enumerate() {
        if function.params.len() > MAX_ARGUMENTS {
            return Err(SemanticError::new(format!(
                "function '{}' has {} parameters; at most {MAX_ARGUMENTS} are supported",
                function.name,
                function.params.len()
            )));
        }
        let signature = Signature {
            id: ir::FunctionId(index),
            arity: function.params.len(),
        };
        if signatures
            .insert(function.name.clone(), signature)
            .is_some()
        {
            return Err(SemanticError::new(format!(
                "duplicate function definition: '{}'",
                function.name
            )));
        }
    }

    match signatures.get("main") {
        None => return Err(SemanticError::new("program must define main()")),
        Some(signature) if signature.arity != 0 => {
            return Err(SemanticError::new("main must have zero parameters"));
        }
        Some(_) => {}
    }

    let mut functions = Vec::with_capacity(program.functions.len());
    for (index, function) in program.functions.into_iter().enumerate() {
        functions
            .push(Resolver::new(&signatures).resolve_function(ir::FunctionId(index), function)?);
    }
    Ok(ir::Program { functions })
}

struct Resolver<'a> {
    signatures: &'a HashMap<String, Signature>,
    scopes: Vec<HashMap<String, ir::LocalId>>,
    next_local: usize,
}

impl<'a> Resolver<'a> {
    fn new(signatures: &'a HashMap<String, Signature>) -> Self {
        Self {
            signatures,
            scopes: vec![HashMap::new()],
            next_local: 0,
        }
    }

    fn resolve_function(
        mut self,
        id: ir::FunctionId,
        function: ast::Function,
    ) -> Result<ir::Function, SemanticError> {
        let mut params = Vec::with_capacity(function.params.len());
        for name in function.params {
            if self.scopes[0].contains_key(&name) {
                return Err(SemanticError::new(format!(
                    "duplicate parameter '{}' in function '{}'",
                    name, function.name
                )));
            }
            let local = self.new_local();
            self.scopes[0].insert(name, local);
            params.push(local);
        }

        let body = self.resolve_block(function.body)?;
        Ok(ir::Function {
            id,
            name: function.name,
            params,
            body,
            local_count: self.next_local,
        })
    }

    fn resolve_block(&mut self, block: ast::Block) -> Result<ir::Block, SemanticError> {
        self.scopes.push(HashMap::new());
        let result: Result<Vec<_>, _> = block
            .statements
            .into_iter()
            .map(|statement| self.resolve_stmt(statement))
            .collect();
        self.scopes.pop();
        Ok(ir::Block {
            statements: result?,
        })
    }

    fn resolve_stmt(&mut self, statement: ast::Stmt) -> Result<ir::Stmt, SemanticError> {
        match statement {
            ast::Stmt::Expr(expr) => Ok(ir::Stmt::Expr(self.resolve_expr(expr)?)),
            ast::Stmt::Return(expr) => Ok(ir::Stmt::Return(self.resolve_expr(expr)?)),
            ast::Stmt::Block(block) => Ok(ir::Stmt::Block(self.resolve_block(block)?)),
            ast::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => Ok(ir::Stmt::If {
                condition: self.resolve_expr(condition)?,
                then_branch: Box::new(self.resolve_stmt(*then_branch)?),
                else_branch: else_branch
                    .map(|branch| self.resolve_stmt(*branch).map(Box::new))
                    .transpose()?,
            }),
            ast::Stmt::While { condition, body } => Ok(ir::Stmt::While {
                condition: self.resolve_expr(condition)?,
                body: Box::new(self.resolve_stmt(*body)?),
            }),
            ast::Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                self.scopes.push(HashMap::new());
                let result = (|| {
                    Ok(ir::Stmt::For {
                        init: init.map(|expr| self.resolve_expr(expr)).transpose()?,
                        condition: condition.map(|expr| self.resolve_expr(expr)).transpose()?,
                        update: update.map(|expr| self.resolve_expr(expr)).transpose()?,
                        body: Box::new(self.resolve_stmt(*body)?),
                    })
                })();
                self.scopes.pop();
                result
            }
            ast::Stmt::Decl { var_type, var_name } => {
                if var_type != "int" {
                    return Err(SemanticError::new(format!(
                        "invalid data type: '{var_type}'"
                    )));
                }
                let ctype = CType::int;
                Ok(ir::Stmt::Decl {
                    ctype,
                    target: match self.find_local_this_scope(&var_name) {
                        Some(_) => {
                            return Err(SemanticError::new(format!(
                                "'{var_name}' is already declared in this scope."
                            )));
                        }
                        None => self.define_local(var_name),
                    },
                })
            }
        }
    }

    fn resolve_expr(&mut self, expr: ast::Expr) -> Result<ir::Expr, SemanticError> {
        match expr {
            ast::Expr::Number(number) => Ok(ir::Expr::Number(number)),
            ast::Expr::Variable(name) => self
                .find_local(&name)
                .map(ir::Expr::Local)
                .ok_or_else(|| SemanticError::new(format!("undefined variable: '{name}'"))),
            ast::Expr::Assign { name, value } => {
                let value = self.resolve_expr(*value)?;
                let target = match self.find_local(&name) {
                    Some(local) => local,
                    None => {
                        return Err(SemanticError::new(format!(
                            "'{name}' is not declared in this scope."
                        )));
                    }
                };
                Ok(ir::Expr::Assign {
                    target,
                    value: Box::new(value),
                })
            }
            ast::Expr::Binary { op, lhs, rhs } => Ok(ir::Expr::Binary {
                op: resolve_binary_op(op),
                lhs: Box::new(self.resolve_expr(*lhs)?),
                rhs: Box::new(self.resolve_expr(*rhs)?),
            }),
            ast::Expr::Call { name, args } => {
                if args.len() > MAX_ARGUMENTS {
                    return Err(SemanticError::new(format!(
                        "call to '{name}' has {} arguments; at most {MAX_ARGUMENTS} are supported",
                        args.len()
                    )));
                }
                let signature = self.signatures.get(&name).copied().ok_or_else(|| {
                    SemanticError::new(format!("call to undefined function: '{name}'"))
                })?;
                if args.len() != signature.arity {
                    return Err(SemanticError::new(format!(
                        "call to '{name}' has {} arguments but {} are required",
                        args.len(),
                        signature.arity
                    )));
                }
                let args = args
                    .into_iter()
                    .map(|arg| self.resolve_expr(arg))
                    .collect::<Result<_, _>>()?;
                Ok(ir::Expr::Call {
                    function: signature.id,
                    args,
                })
            }
        }
    }

    fn new_local(&mut self) -> ir::LocalId {
        let local = ir::LocalId(self.next_local);
        self.next_local += 1;
        local
    }

    fn define_local(&mut self, name: String) -> ir::LocalId {
        let local = self.new_local();
        self.scopes
            .last_mut()
            .expect("a resolver always has a scope")
            .insert(name, local);
        local
    }

    fn find_local(&self, name: &str) -> Option<ir::LocalId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn find_local_this_scope(&self, name: &str) -> Option<ir::LocalId> {
        self.scopes
            .last()
            .and_then(|scope| scope.get(name).copied())
    }
}

fn resolve_binary_op(op: ast::BinaryOp) -> ir::BinaryOp {
    match op {
        ast::BinaryOp::Add => ir::BinaryOp::Add,
        ast::BinaryOp::Sub => ir::BinaryOp::Sub,
        ast::BinaryOp::Mul => ir::BinaryOp::Mul,
        ast::BinaryOp::Div => ir::BinaryOp::Div,
        ast::BinaryOp::Less => ir::BinaryOp::Less,
        ast::BinaryOp::LessEqual => ir::BinaryOp::LessEqual,
        ast::BinaryOp::Equal => ir::BinaryOp::Equal,
        ast::BinaryOp::NotEqual => ir::BinaryOp::NotEqual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze_source(source: &str) -> Result<ir::Program, SemanticError> {
        let tokens = crate::lexer::lex(source).unwrap();
        analyze(crate::parser::parse(&tokens).unwrap())
    }

    fn error(source: &str) -> String {
        analyze_source(source).unwrap_err().to_string()
    }

    #[test]
    fn var_declare() {
        let program = analyze_source("main(){int a; a=10; {int a; a = 20;} return a;}").unwrap();
        assert_eq!(program.functions[0].local_count, 2);
    }

    #[test]
    fn reject_var_double_declare() {
        let src = "main(){int a; a=10; {int a; a = 20;} int a; return a;}";
        assert!(error(src).contains("is already declared"));
    }

    #[test]
    fn reject_undeclared() {
        let src = "main(){a=10; {int a; a = 20;} int a; return a;}";
        assert!(error(src).contains("is not declared"));
    }

    #[test]
    fn reject_out_of_scope() {
        let src = "main(){int a; a=10; {int b; a = 20;} b = 30; return a;}";
        assert!(error(src).contains("is not declared"));
    }

    #[test]
    fn resolves_forward_calls_recursion_and_eight_arguments() {
        let program = analyze_source(
            "main() { return sum8(1,2,3,4,5,6,7,8); }
             sum8(a,b,c,d,e,f,g,h) { if (a == 0) return 0; return a+b+c+d+e+f+g+h; }",
        )
        .unwrap();

        assert_eq!(program.functions.len(), 2);
        assert_eq!(program.functions[1].params.len(), 8);
        assert_eq!(program.functions[1].local_count, 8);
    }

    #[test]
    fn rejects_invalid_function_definitions_and_calls() {
        assert!(error("main() {} main() {}").contains("duplicate function"));
        assert!(error("main() {} f(a,a) {}").contains("duplicate parameter"));
        assert!(error("other() {}").contains("define main"));
        assert!(error("main(a) {}").contains("zero parameters"));
        assert!(error("main() { f(); } f(a) {}").contains("1 are required"));
        assert!(error("main() { missing(); }").contains("undefined function"));
        assert!(error("main() { f(1,2,3,4,5,6,7,8,9); } f() {}").contains("at most 8"));
    }

    #[test]
    fn requires_declaration_before_use() {
        assert!(error("main() { return a; }").contains("undefined variable"));
        assert!(error("main() { a = a + 1; }").contains("undefined variable"));
        analyze_source("main() { int a; a = 1; return a; }").unwrap();
    }

    #[test]
    fn resolves_outer_assignments_and_separates_sibling_locals() {
        let program = analyze_source(
            "main() { int outer; outer = 1; { int sibling; outer = 2; sibling = 3; } { int sibling; sibling = 4; } return outer; }",
        )
        .unwrap();
        let function = &program.functions[0];
        assert_eq!(function.local_count, 3);
        assert!(
            error("main() { { int local; local = 1; } return local; }")
                .contains("undefined variable")
        );
    }

    #[test]
    fn for_has_its_own_scope() {
        analyze_source("main() { for (; 0;) int i; return 0; }").unwrap();
        assert!(error("main() { for (; 0;) int i; return i; }").contains("undefined variable"));
    }
}
