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
                    body: func.body.clone(),
                });

                self.env.define(func.name.clone(), value.clone());
                value
            }

            Node::Block(block) => {
                self.env.push_scope();

                let mut last = Value::Void;

                for segment in &block.segments {
                    for n in &segment.nodes {
                        last = self.eval_value(n);
                    }
                }

                self.env.pop_scope();
                last
            }

            Node::Call(call) => {
            // Resolve the callable before entering the function scope.
            let callee = self.eval_value(call.callee.as_ref());

            let Value::Func(func) = callee else {
                panic!("attempted to call a non-function value");
            };

            // Explicit arguments are evaluated in source order in the caller's scope.
            let argument_values: Vec<Value> = call
                .args
                .iter()
                .map(|argument| self.eval_value(argument))
                .collect();

            if argument_values.len() > func.params.len() {
                panic!(
                    "function `{}` expected at most {} arguments but received {}",
                    func.name,
                    func.params.len(),
                    argument_values.len()
                );
            }

            self.env.push_scope();

            // Bind parameters from left to right.
            for (index, param) in func.params.iter().enumerate() {
                let value = match argument_values.get(index) {
                    Some(value) => value.clone(),

                    None => match &param.default {
                        Some(default) => self.eval_value(default),

                        None => {
                            self.env.pop_scope();

                            panic!(
                                "function `{}` is missing required argument `{}`",
                                func.name,
                                param.name
                            );
                        }
                    },
                };

                self.env.define(param.name.clone(), value);
            }

            let mut result = Value::Void;

            for node in &func.body {
                match self.eval_node_ctrl(node) {
                    Control::Continue => {}

                    Control::Return(value) => {
                        result = value;
                        break;
                    }
                }
            }

            self.env.pop_scope();
            result
        }

        Node::Add(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            match (lhs, rhs) {
                (Value::Num(a), Value::Num(b)) => Value::Num(a + b),
                _ => panic!("addition requires two numbers"),
            }
        }

        Node::Sub(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            match (lhs, rhs) {
                (Value::Num(a), Value::Num(b)) => Value::Num(a - b),
                _ => panic!("subtraction requires two numbers"),
            }
        }

        Node::Mul(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            match (lhs, rhs) {
                (Value::Num(a), Value::Num(b)) => Value::Num(a * b),
                _ => panic!("multiplication requires two numbers"),
            }
        }

        Node::Div(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            match (lhs, rhs) {
                (Value::Num(_), Value::Num(0)) => panic!("division by zero"),
                (Value::Num(a), Value::Num(b)) => Value::Num(a / b),
                _ => panic!("division requires two numbers"),
            }
        }

        Node::Mod(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            match (lhs, rhs) {
                (Value::Num(_), Value::Num(0)) => panic!("modulo by zero"),
                (Value::Num(a), Value::Num(b)) => Value::Num(a % b),
                _ => panic!("modulo requires two numbers"),
            }
        }

        Node::Eq(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            Value::Flag(lhs == rhs)
        }

        Node::Ne(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            Value::Flag(lhs != rhs)
        }

        Node::Lt(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            match (lhs, rhs) {
                (Value::Num(a), Value::Num(b)) => Value::Flag(a < b),
                _ => panic!("less-than comparison requires two numbers"),
            }
        }

        Node::Le(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            match (lhs, rhs) {
                (Value::Num(a), Value::Num(b)) => Value::Flag(a <= b),
                _ => panic!("less-than-or-equal comparison requires two numbers"),
            }
        }

        Node::Gt(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            match (lhs, rhs) {
                (Value::Num(a), Value::Num(b)) => Value::Flag(a > b),
                _ => panic!("greater-than comparison requires two numbers"),
            }
        }

        Node::Ge(lhs, rhs) => {
            let lhs = self.eval_value(lhs);
            let rhs = self.eval_value(rhs);

            match (lhs, rhs) {
                (Value::Num(a), Value::Num(b)) => Value::Flag(a >= b),
                _ => panic!("greater-than-or-equal comparison requires two numbers"),
            }
        }

        Node::And(lhs, rhs) => {
            let lhs = self.eval_value(lhs);

            if truth_of(&lhs) == Truth::False {
                Value::Flag(false)
            } else {
                let rhs = self.eval_value(rhs);
                Value::Flag(truth_of(&rhs) == Truth::True)
            }
        }

        Node::Or(lhs, rhs) => {
            let lhs = self.eval_value(lhs);

            if truth_of(&lhs) == Truth::True {
                Value::Flag(true)
            } else {
                let rhs = self.eval_value(rhs);
                Value::Flag(truth_of(&rhs) == Truth::True)
            }
        }

        Node::Not(value) => {
            let value = self.eval_value(value);

            Value::Flag(truth_of(&value) == Truth::False)
        }

        Node::Neg(value) => {
            let value = self.eval_value(value);

            match value {
                Value::Num(number) => Value::Num(-number),
                _ => panic!("numeric negation requires a number"),
            }
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
                    let v = self.eval_value(&branch.expr);
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

                for segment in &block.segments {
                    for n in &segment.nodes {
                        let ctl = self.eval_node_ctrl(n);
                        if let Control::Return(v) = ctl {
                            self.env.pop_scope();
                            return Control::Return(v);
                        }
                    }
                }

                self.env.pop_scope();
                Control::Continue
            }

            Node::Func(func) => {
                let value = Value::Func(crate::compiler::semantics::value::Func {
                    name: func.name.clone(),
                    params: func.params.clone(),
                    body: func.body.clone(),
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
