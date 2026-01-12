use crate::compiler::ast::{Expr, Stmt};
use super::Value;

pub struct Evaluator {
    // later: scopes, environment, heap, etc.
}

impl Evaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Lit(lit) => lit.into(),
            _ => todo!(),
        }
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            _ => todo!(),
        }
    }
}

impl From<&Literal> for Value {
    fn from(l: &Literal) -> Self {
        match l {
            Literal::Num(n) => Value::Num(*n),
            Literal::Dec(s) => Value::Dec(s.clone()),
            Literal::Flag(b) => Value::Flag(*b),
            Literal::Text(t) => Value::Text(t.clone()),
            Literal::Emp => Value::Emp,
        }
    }
}

