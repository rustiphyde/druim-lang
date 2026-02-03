use crate::compiler::ast::{Node, Program};
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
        for node in &program.nodes {
            self.eval_node(node);
        }
    }

    /// For tests only (read current value).
    pub fn get(&self, name: &str) -> Option<Value> {
        self.env.get_value(name)
    }

    fn eval_value(&mut self, node: &Node) -> Value {
        match node {
            Node::Lit(lit) => Value::from_literal(lit),

            Node::Ident(name) => {
                self.env.get_value(name).unwrap_or(Value::Void)
            }

            Node::Func(func) => {
                let value = Value::Func(crate::compiler::semantics::value::Func {
                    name: func.name.clone(),
                    params: func.params.clone(),
                    bodies: func.bodies.clone(),
                });

                self.env.define(func.name.clone(), value.clone());
                value
            }

            Node::Block(block) => {
                self.env.push_scope();

                let mut last = Value::Void;
                for n in &block.nodes {
                    last = self.eval_value(n);
                }

                self.env.pop_scope();
                last
            }

            _ => Value::Void,
        }
    }


    pub fn eval_node(&mut self, node: &Node) {
        match self.eval_node_ctrl(node) {
            Control::Continue => {}
            Control::Return(_) => {
                panic!("return executed outside of a function");
            }
        }
    }


    fn eval_node_ctrl(&mut self, node: &Node) -> Control {
        match node {
            Node::Define(def) => {
                let v = self.eval_value(&def.value);
                self.env.define(def.name.clone(), v);
                Control::Continue
            }

            Node::DefineEmpty(def) => {
                self.env.define(def.name.clone(), Value::Void);
                Control::Continue
            }

            Node::Copy(copy) => {
                self.env
                    .copy(copy.name.clone(), &copy.target)
                    .expect("copy target must exist");
                Control::Continue
            }

            Node::Bind(bind) => {
                let v = self
                    .env
                    .get_value(&bind.target)
                    .expect("bind target must exist");
                self.env.define(bind.name.clone(), v);
                Control::Continue
            }

            Node::Guard(guard) => {
                let mut result = Value::Void;

                for branch in &guard.branches {
                    let v = self.eval_value(branch);
                    if truth_of(&v) == Truth::True {
                        result = v;
                        break;
                    }
                }

                self.env.define(guard.target.clone(), result);
                Control::Continue
            }

            Node::Ret(ret) => {
                let v = match &ret.value {
                    Some(node) => self.eval_value(node),
                    None => Value::Void,
                };
                Control::Return(v)
            }

            Node::Block(block) => {
                self.env.push_scope();

                for n in &block.nodes {
                    let ctl = self.eval_node_ctrl(n);
                    if let Control::Return(v) = ctl {
                        self.env.pop_scope();
                        return Control::Return(v);
                    }
                }

                self.env.pop_scope();
                Control::Continue
            }

            Node::Func(func) => {
                let value = Value::Func(crate::compiler::semantics::value::Func {
                    name: func.name.clone(),
                    params: func.params.clone(),
                    bodies: func.bodies.clone(),
                });

                self.env.define(func.name.clone(), value.clone());
                Control::Continue
            }

            // literals, identifiers, calls, etc.
           other => {
                let _ = self.eval_value(other);
                Control::Continue
            }

        }

    }
}
