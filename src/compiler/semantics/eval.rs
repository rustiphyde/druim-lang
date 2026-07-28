use crate::compiler::ast::{Node, Program};
use crate::compiler::semantics::env::Env;
use crate::compiler::semantics::truth::{truth_of, Truth};
use crate::compiler::semantics::value::Value;
use crate::compiler::error::{Diagnostic, Span};

pub struct Evaluator {
    env: Env,
}

#[derive(Debug, Clone, PartialEq)]
enum Control {
    Continue,
    Return(Value),
}

fn runtime_span() -> Span {
    Span {
        start: 0,
        end: 0,
    }
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            env: Env::new(),
        }
    }

    pub fn eval_program(
        &mut self,
        program: &Program,
    ) -> Result<(), Diagnostic> {
        for node in &program.nodes {
            self.eval_node(node)?;
        }

        Ok(())
    }

    /// For tests only (read current value).
    pub fn get(&self, name: &str) -> Option<Value> {
        self.env.get_value(name)
    }

    fn eval_value(&mut self, node: &Node) -> Result<Value, Diagnostic> {
        
        match node {
            Node::Lit(lit) => {
                Ok(Value::from_literal(lit))
            }

            Node::Box(box_literal) => {
                let values = box_literal
                    .values
                    .iter()
                    .map(|value| self.eval_value(value))
                    .collect::<Result<Vec<Value>, Diagnostic>>()?;

                Ok(Value::Box(values))
            }

            Node::Bag(bag_literal) => {
                let mut entries = std::collections::HashMap::with_capacity(
                    bag_literal.entries.len(),
                );

                for entry in &bag_literal.entries {
                    let value = self.eval_value(&entry.value)?;
                    entries.insert(entry.name.clone(), value);
                }

                Ok(Value::Bag(entries))
            }

            Node::Ident(name) => {
                self.get(name).ok_or_else(|| {
                    Diagnostic::error(
                        format!("undeclared identifier `{name}`"),
                        runtime_span(),
                    )
                })
            }

            Node::Func(func) => {
                let value = Value::Func(
                    crate::compiler::semantics::value::Func {
                        name: func.name.clone(),
                        params: func.params.clone(),
                        body: func.body.clone(),
                    },
                );

                self.env.define(func.name.clone(), value.clone());

                Ok(value)
            }

            Node::Block(block) => {
                self.env.push_scope();

                let result = (|| {
                    let mut last = Value::Void;

                    for segment in &block.segments {
                        for node in &segment.nodes {
                            last = self.eval_value(node)?;
                        }
                    }

                    Ok(last)
                })();

                self.env.pop_scope();

                result
            }

            Node::Call(call) => {
                let callee = self.eval_value(call.callee.as_ref())?;

                let Value::Func(func) = callee else {
                    return Err(Diagnostic::error(
                        "attempted to call a non-function value",
                        runtime_span(),
                    ));
                };

                let argument_values = call
                    .args
                    .iter()
                    .map(|argument| self.eval_value(argument))
                    .collect::<Result<Vec<Value>, Diagnostic>>()?;

                if argument_values.len() > func.params.len() {
                    return Err(Diagnostic::error(
                        format!(
                            "function `{}` expected at most {} arguments but received {}",
                            func.name,
                            func.params.len(),
                            argument_values.len(),
                        ),
                        runtime_span(),
                    ));
                }

                self.env.push_scope();

                let result = (|| {
                    for (index, param) in func.params.iter().enumerate() {
                        let value = match argument_values.get(index) {
                            Some(value) => value.clone(),

                            None => match &param.default {
                                Some(default) => self.eval_value(default)?,

                                None => {
                                    return Err(Diagnostic::error(
                                        format!(
                                            "function `{}` is missing required argument `{}`",
                                            func.name,
                                            param.name,
                                        ),
                                        runtime_span(),
                                    ));
                                }
                            },
                        };

                        self.env.define(param.name.clone(), value);
                    }

                    let mut result = Value::Void;

                    for node in &func.body {
                        match self.eval_node_ctrl(node)? {
                            Control::Continue => {}

                            Control::Return(value) => {
                                result = value;
                                break;
                            }
                        }
                    }

                    Ok(result)
                })();

                self.env.pop_scope();

                result
            }

           Node::Add(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),

                    _ => Err(Diagnostic::error(
                        "addition requires two numbers",
                        runtime_span(),
                    )),
                }
            }

            Node::Sub(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a - b)),

                    _ => Err(Diagnostic::error(
                        "subtraction requires two numbers",
                        runtime_span(),
                    )),
                }
            }

            Node::Mul(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a * b)),

                    _ => Err(Diagnostic::error(
                        "multiplication requires two numbers",
                        runtime_span(),
                    )),
                }
            }

            Node::Div(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(_), Value::Num(0)) => Err(
                        Diagnostic::error(
                            "division by zero",
                            runtime_span(),
                        ),
                    ),

                    (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a / b)),

                    _ => Err(Diagnostic::error(
                        "division requires two numbers",
                        runtime_span(),
                    )),
                }
            }

            Node::Mod(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(_), Value::Num(0)) => Err(
                        Diagnostic::error(
                            "modulo by zero",
                            runtime_span(),
                        ),
                    ),

                    (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a % b)),

                    _ => Err(Diagnostic::error(
                        "modulo requires two numbers",
                        runtime_span(),
                    )),
                }
            }

            Node::Eq(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                Ok(Value::Flag(lhs == rhs))
            }

            Node::Ne(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                Ok(Value::Flag(lhs != rhs))
            }

            Node::Lt(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Flag(a < b))
                    }

                    _ => Err(Diagnostic::error(
                        "less-than comparison requires two numbers",
                        runtime_span(),
                    )),
                }
            }

            Node::Le(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Flag(a <= b))
                    }

                    _ => Err(Diagnostic::error(
                        "less-than-or-equal comparison requires two numbers",
                        runtime_span(),
                    )),
                }
            }

            Node::Gt(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Flag(a > b))
                    }

                    _ => Err(Diagnostic::error(
                        "greater-than comparison requires two numbers",
                        runtime_span(),
                    )),
                }
            }

            Node::Ge(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Flag(a >= b))
                    }

                    _ => Err(Diagnostic::error(
                        "greater-than-or-equal comparison requires two numbers",
                        runtime_span(),
                    )),
                }
            }

            Node::And(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;

                if truth_of(&lhs) == Truth::False {
                    Ok(Value::Flag(false))
                } else {
                    let rhs = self.eval_value(rhs)?;
                    Ok(Value::Flag(
                        truth_of(&rhs) == Truth::True,
                    ))
                }
            }

            Node::Or(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;

                if truth_of(&lhs) == Truth::True {
                    Ok(Value::Flag(true))
                } else {
                    let rhs = self.eval_value(rhs)?;
                    Ok(Value::Flag(
                        truth_of(&rhs) == Truth::True,
                    ))
                }
            }

            Node::Not(value) => {
                let value = self.eval_value(value)?;

                Ok(Value::Flag(
                    truth_of(&value) == Truth::False,
                ))
            }

            Node::Neg(value) => {
                let value = self.eval_value(value)?;

                match value {
                    Value::Num(number) => {
                        Ok(Value::Num(-number))
                    }

                    _ => Err(Diagnostic::error(
                        "numeric negation requires a number",
                        runtime_span(),
                    )),
                }
            }

            Node::Get(lhs, selector) => {
                let lhs = self.eval_value(lhs)?;

                match lhs {
                    Value::Box(values) => {
                        let Node::Index(index_expr) = selector.as_ref() else {
                            return Err(Diagnostic::error(
                                "Box traversal requires an indexed selector",
                                runtime_span(),
                            ));
                        };

                        let index = self.eval_value(index_expr)?;

                        match index {
                            Value::Num(index) if index >= 0 => Ok(
                                values
                                    .get(index as usize)
                                    .cloned()
                                    .unwrap_or(Value::Void),
                            ),

                            Value::Num(_) => Err(Diagnostic::error(
                                "Box index cannot be negative",
                                runtime_span(),
                            )),

                            _ => Err(Diagnostic::error(
                                "Box index must evaluate to a number",
                                runtime_span(),
                            )),
                        }
                    }

                    Value::Bag(entries) => {
                        let Node::Ident(name) = selector.as_ref() else {
                            return Err(Diagnostic::error(
                                "Bag traversal requires a named selector",
                                runtime_span(),
                            ));
                        };

                        Ok(
                            entries
                                .get(name)
                                .cloned()
                                .unwrap_or(Value::Void),
                        )
                    }

                    _ => Ok(Value::Void),
                }
            }

           Node::Has(lhs, selector) => {
                let lhs = self.eval_value(lhs)?;

                match lhs {
                    Value::Box(values) => {
                        let Node::Index(index_expr) = selector.as_ref() else {
                            return Err(Diagnostic::error(
                                "Box traversal requires an indexed selector",
                                runtime_span(),
                            ));
                        };

                        let index = self.eval_value(index_expr)?;

                        match index {
                            Value::Num(index) if index >= 0 => {
                                Ok(Value::Flag((index as usize) < values.len()))
                            }

                            Value::Num(_) => Err(Diagnostic::error(
                                "Box index cannot be negative",
                                runtime_span(),
                            )),

                            _ => Err(Diagnostic::error(
                                "Box index must evaluate to a number",
                                runtime_span(),
                            )),
                        }
                    }

                    Value::Bag(entries) => {
                        let Node::Ident(name) = selector.as_ref() else {
                            return Err(Diagnostic::error(
                                "Bag traversal requires a named selector",
                                runtime_span(),
                            ));
                        };

                        Ok(Value::Flag(entries.contains_key(name)))
                    }

                    _ => Ok(Value::Flag(false)),
                }
            }

           _ => Ok(Value::Void),
        }
    }


    pub fn eval_node(
        &mut self,
        node: &Node,
    ) -> Result<(), Diagnostic> {
        match self.eval_node_ctrl(node)? {
            Control::Continue => Ok(()),

            Control::Return(_) => Err(
                Diagnostic::error(
                    "return executed outside of a function",
                    runtime_span(),
                ),
            ),
        }
    }

    fn eval_node_ctrl(
        &mut self,
        node: &Node,
    ) -> Result<Control, Diagnostic> {
        match node {
           Node::Define(def) => {
                let value = self.eval_value(&def.value)?;
                self.env.define(def.name.clone(), value);

                Ok(Control::Continue)
            }

            Node::DefineEmpty(def) => {
                self.env.define(def.name.clone(), Value::Void);

                Ok(Control::Continue)
            }

            Node::Copy(copy) => {
                self.env
                    .copy(copy.name.clone(), &copy.target)
                    .map_err(|_| {
                        Diagnostic::error(
                            "copy target must exist",
                            runtime_span(),
                        )
                    })?;

                Ok(Control::Continue)
            }

            Node::Bind(bind) => {
                let value = self
                    .env
                    .get_value(&bind.target)
                    .ok_or_else(|| {
                        Diagnostic::error(
                            "bind target must exist",
                            runtime_span(),
                        )
                    })?;

                self.env.define(bind.name.clone(), value);

                Ok(Control::Continue)
            }

            Node::Guard(guard) => {
                let mut result = Value::Void;

                for branch in &guard.branches {
                    let value = self.eval_value(&branch.expr)?;

                    if truth_of(&value) == Truth::True {
                        result = value;
                        break;
                    }
                }

                self.env.define(guard.target.clone(), result);

                Ok(Control::Continue)
            }

            Node::Ret(ret) => {
                let value = match &ret.value {
                    Some(node) => self.eval_value(node)?,
                    None => Value::Void,
                };

                Ok(Control::Return(value))
            }

            Node::Block(block) => {
                self.env.push_scope();

                let result = (|| {
                    for segment in &block.segments {
                        for node in &segment.nodes {
                            let control = self.eval_node_ctrl(node)?;

                            if let Control::Return(value) = control {
                                return Ok(Control::Return(value));
                            }
                        }
                    }

                    Ok(Control::Continue)
                })();

                self.env.pop_scope();

                result
            }

            Node::Func(func) => {
                let value = Value::Func(
                    crate::compiler::semantics::value::Func {
                        name: func.name.clone(),
                        params: func.params.clone(),
                        body: func.body.clone(),
                    },
                );

                self.env.define(func.name.clone(), value);

                Ok(Control::Continue)
            }

            other => {
                let _ = self.eval_value(other)?;

                Ok(Control::Continue)
            }

        }

    }
}
