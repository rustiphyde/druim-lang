use crate::compiler::ast::{Expr, Stmt, Program};
use crate::compiler::semantics::env::Env;
use crate::compiler::semantics::truth::{truth_of, Truth};
use crate::compiler::semantics::value::Value;

pub struct Evaluator {
    env: Env,
}

#[derive(Debug, Clone, PartialEq)]
enum Control {
    Continue,
    Return(Value),
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

            Expr::FnBlock { name, args, bodies } => {
                let func = crate::compiler::semantics::value::Function {
                    name: name.clone(),
                    params: args.iter().map(|p| p.name.clone()).collect(),
                    bodies: bodies.clone(),
                };

                let value = Value::Func(func);

                // Bind function into the current scope
                self.env.define(name.clone(), value.clone());

                value
            }

            _ => todo!("expression evaluation not implemented yet"),
        }
    }


    pub fn eval_stmt(&mut self, stmt: &Stmt) {
        match self.eval_stmt_ctrl(stmt) {
            Control::Continue => {}
            Control::Return(_v) => {
                // For now, this is a runtime error because we have not
                // implemented "ret only allowed inside functions" as a compile-time rule yet.
                panic!("return executed outside of a function");
            }
        }    }

    fn eval_stmt_ctrl(&mut self, stmt: &Stmt) -> Control {
        match stmt {
            Stmt::Define { name, value } => {
                let v = self.eval_expr(value);
                self.env.define(name.clone(), v);
                Control::Continue
            }

            Stmt::DefineEmpty { name } => {
                // Use your current "void" value here.
                // If your Value type is still Emp in this file, keep Value::Emp.
                self.env.define(name.clone(), Value::Void);
                Control::Continue
            }

            Stmt::Bind { name, target } => {
                self.env
                    .bind(name.clone(), target)
                    .expect("bind target must exist");
                Control::Continue
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

                Control::Continue
            }

            Stmt::Guard { target, branches } => {
                // Use your current "void" default here.
                let mut result = Value::Void;

                for expr in branches {
                    let v = self.eval_expr(expr);
                    if truth_of(&v) == Truth::True {
                        result = v;
                        break;
                    }
                }

                self.env.define(target.clone(), result);
                Control::Continue
            }

            Stmt::Return { value } => {
                let v = match value {
                    Some(expr) => self.eval_expr(expr),
                    None => Value::Void,
                };
                Control::Return(v)
            }

            Stmt::Block { stmts } => {
                self.env.push_scope();

                for s in stmts {
                    let ctl = self.eval_stmt_ctrl(s);
                    if let Control::Return(v) = ctl {
                        self.env.pop_scope();
                        return Control::Return(v);
                    }
                }

                self.env.pop_scope();
                Control::Continue
            }

            Stmt::SendTo { .. } => {
                todo!("send semantics not implemented yet");
            }
        }
    }
}
