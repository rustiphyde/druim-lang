use crate::compiler::ast::{Expr, Stmt, Program};
use crate::compiler::semantics::env::Env;
use crate::compiler::semantics::truth::{truth_of, Truth};
use crate::compiler::semantics::value::Value;

pub struct Evaluator {
    env: Env,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            env: Env::new(),
        }
    }

    pub fn eval_program(&mut self, program: &Program) {
        for stmt in &program.stmts {
            self.eval_stmt(stmt);
        }
    }

    /// For tests only (read current value).
    pub fn get(&self, name: &str) -> Option<Value> {
        self.env.get_value(name)
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Lit(lit) => Value::from_literal(lit),

            Expr::Ident(name) => {
                self.env
                    .get_value(name)
                    .unwrap_or(Value::Void)
            }

            _ => todo!("expression evaluation not implemented yet"),
        }
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Define { name, value } => {
                let v = self.eval_expr(value);
                self.env.define(name.clone(), v);
            }

            Stmt::DefineEmpty { name } => {
                self.env.define(name.clone(), Value::Void);
            }

            Stmt::Bind { name, target } => {
                self.env
                    .bind(name.clone(), target)
                    .expect("bind target must exist");
            }

            Stmt::AssignFrom { target, source } => {
                let value = self.eval_expr(source);

                if let Expr::Ident(name) = target {
                    self.env
                        .assign(name, value)
                        .expect("assignment target must exist");
                } else {
                    panic!("invalid assignment target");
                }
            }

            Stmt::Guard { target, branches } => {
                let mut result = Value::Void;

                for expr in branches {
                    let v = self.eval_expr(expr);
                    if truth_of(&v) == Truth::True {
                        result = v;
                        break;
                    }
                }

                self.env.define(target.clone(), result);
            }

            Stmt::Block { stmts } => {
                self.env.push_scope();
                for s in stmts {
                    self.eval_stmt(s);
                }
                self.env.pop_scope();
            }

            Stmt::SendTo { .. } => {
                todo!("send semantics not implemented yet");
            }
        }
    }
}
