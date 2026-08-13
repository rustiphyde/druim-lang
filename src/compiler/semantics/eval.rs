use crate::compiler::ast::{Node, NodeKind, Program};
use crate::compiler::semantics::env::{
    Env,
    EnvError,
    GlobalBindError,
    GlobalCopyError,
    DefineError,
    CopyError,
    BindError,
};
use crate::compiler::semantics::truth::{truth_of, Truth};
use crate::compiler::semantics::value::Value;
use crate::compiler::error::Diagnostic;

pub struct Evaluator {
    env: Env,
}

#[derive(Debug, Clone, PartialEq)]
enum Control {
    Continue,
    Return(Value),
}

fn local_name(node: &Node) -> Option<&str> {
    match &node.kind {
        NodeKind::Define(def) => Some(&def.name),
        NodeKind::DefineEmpty(def) => Some(&def.name),
        NodeKind::Copy(copy) => Some(&copy.name),
        NodeKind::Bind(bind) => Some(&bind.name),
        NodeKind::Guard(guard) => Some(&guard.target),
        NodeKind::Func(func) => Some(&func.name),
        _ => None,
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
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
        match &node.kind {
            NodeKind::Lit(lit) => {
                Ok(Value::from_literal(lit))
            }

            NodeKind::Box(box_literal) => {
                let values = box_literal
                    .values
                    .iter()
                    .map(|value| self.eval_value(value))
                    .collect::<Result<Vec<Value>, Diagnostic>>()?;

                Ok(Value::Box(values))
            }

            NodeKind::Bag(bag_literal) => {
                let mut entries = std::collections::HashMap::with_capacity(
                    bag_literal.entries.len(),
                );

                for entry in &bag_literal.entries {
                    let value = self.eval_value(&entry.value)?;
                    entries.insert(entry.name.clone(), value);
                }

                Ok(Value::Bag(entries))
            }

            NodeKind::Ident(name) => {
                self.get(name).ok_or_else(|| {
                    Diagnostic::error(
                        format!("undeclared identifier `{name}`"),
                        node.span,
                    )
                })
            }

            NodeKind::Func(func) => {
                let value = Value::Func(
                    crate::compiler::semantics::value::Func {
                        name: func.name.clone(),
                        params: func.params.clone(),
                        body: func.body.clone(),
                    },
                );

                self.env
                    .define(func.name.clone(), value.clone())
                    .map_err(|error| {
                        match error {
                            DefineError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(value)
            }

            NodeKind::Call(call) => {
                let callee = self.eval_value(call.callee.as_ref())?;

                let Value::Func(func) = callee else {
                    return Err(Diagnostic::error(
                        "attempted to call a non-function value",
                        call.callee.span,
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
                        node.span,
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
                                        node.span,
                                    ));
                                }
                            },
                        };

                        self.env
                            .define(param.name.clone(), value)
                            .map_err(|error| {
                                match error {
                                    DefineError::AlreadyDefined(name) => {
                                        Diagnostic::error(
                                            format!("identifier `{name}` is already defined"),
                                            node.span,
                                        )
                                    }
                                }
                            })?;
                    }

                    let mut result = Value::Void;

                    for body_node in &func.body {
                        match self.eval_node_ctrl(body_node)? {
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

            NodeKind::Add(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Num(a + b))
                    }

                    _ => Err(Diagnostic::error(
                        "addition requires two numbers",
                        node.span,
                    )),
                }
            }

            NodeKind::Sub(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Num(a - b))
                    }

                    _ => Err(Diagnostic::error(
                        "subtraction requires two numbers",
                        node.span,
                    )),
                }
            }

            NodeKind::Mul(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Num(a * b))
                    }

                    _ => Err(Diagnostic::error(
                        "multiplication requires two numbers",
                        node.span,
                    )),
                }
            }

            NodeKind::Div(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs_value = self.eval_value(rhs)?;

                match (lhs, rhs_value) {
                    (Value::Num(_), Value::Num(0)) => {
                        Err(Diagnostic::error(
                            "division by zero",
                            rhs.span,
                        ))
                    }

                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Num(a / b))
                    }

                    _ => Err(Diagnostic::error(
                        "division requires two numbers",
                        node.span,
                    )),
                }
            }

            NodeKind::Mod(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs_value = self.eval_value(rhs)?;

                match (lhs, rhs_value) {
                    (Value::Num(_), Value::Num(0)) => {
                        Err(Diagnostic::error(
                            "modulo by zero",
                            rhs.span,
                        ))
                    }

                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Num(a % b))
                    }

                    _ => Err(Diagnostic::error(
                        "modulo requires two numbers",
                        node.span,
                    )),
                }
            }

            NodeKind::Eq(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                Ok(Value::Flag(lhs == rhs))
            }

            NodeKind::Ne(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                Ok(Value::Flag(lhs != rhs))
            }

            NodeKind::Lt(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Flag(a < b))
                    }

                    _ => Err(Diagnostic::error(
                        "less-than comparison requires two numbers",
                        node.span,
                    )),
                }
            }

            NodeKind::Le(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Flag(a <= b))
                    }

                    _ => Err(Diagnostic::error(
                        "less-than-or-equal comparison requires two numbers",
                        node.span,
                    )),
                }
            }

            NodeKind::Gt(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Flag(a > b))
                    }

                    _ => Err(Diagnostic::error(
                        "greater-than comparison requires two numbers",
                        node.span,
                    )),
                }
            }

            NodeKind::Ge(lhs, rhs) => {
                let lhs = self.eval_value(lhs)?;
                let rhs = self.eval_value(rhs)?;

                match (lhs, rhs) {
                    (Value::Num(a), Value::Num(b)) => {
                        Ok(Value::Flag(a >= b))
                    }

                    _ => Err(Diagnostic::error(
                        "greater-than-or-equal comparison requires two numbers",
                        node.span,
                    )),
                }
            }

            NodeKind::And(lhs, rhs) => {
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

            NodeKind::Or(lhs, rhs) => {
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

            NodeKind::Not(value) => {
                let value = self.eval_value(value)?;

                Ok(Value::Flag(
                    truth_of(&value) == Truth::False,
                ))
            }

            NodeKind::Neg(value) => {
                let evaluated = self.eval_value(value)?;

                match evaluated {
                    Value::Num(number) => {
                        Ok(Value::Num(-number))
                    }

                    _ => Err(Diagnostic::error(
                        "numeric negation requires a number",
                        node.span,
                    )),
                }
            }

            NodeKind::Get(lhs, selector) => {
                let lhs = self.eval_value(lhs)?;

                match lhs {
                    Value::Box(values) => {
                        let NodeKind::Index(index_expr) = &selector.kind else {
                            return Err(Diagnostic::error(
                                "Box traversal requires an indexed selector",
                                selector.span,
                            ));
                        };

                        let index = self.eval_value(index_expr)?;

                        match index {
                            Value::Num(index) if index >= 0 => {
                                Ok(
                                    values
                                        .get(index as usize)
                                        .cloned()
                                        .unwrap_or(Value::Void),
                                )
                            }

                            Value::Num(_) => {
                                Err(Diagnostic::error(
                                    "Box index cannot be negative",
                                    index_expr.span,
                                ))
                            }

                            _ => {
                                Err(Diagnostic::error(
                                    "Box index must evaluate to a number",
                                    index_expr.span,
                                ))
                            }
                        }
                    }

                    Value::Bag(entries) => {
                        let NodeKind::Ident(name) = &selector.kind else {
                            return Err(Diagnostic::error(
                                "Bag traversal requires a named selector",
                                selector.span,
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

            NodeKind::Has(lhs, selector) => {
                let lhs = self.eval_value(lhs)?;

                match lhs {
                    Value::Box(values) => {
                        let NodeKind::Index(index_expr) = &selector.kind else {
                            return Err(Diagnostic::error(
                                "Box traversal requires an indexed selector",
                                selector.span,
                            ));
                        };

                        let index = self.eval_value(index_expr)?;

                        match index {
                            Value::Num(index) if index >= 0 => {
                                Ok(Value::Flag(
                                    (index as usize) < values.len(),
                                ))
                            }

                            Value::Num(_) => {
                                Err(Diagnostic::error(
                                    "Box index cannot be negative",
                                    index_expr.span,
                                ))
                            }

                            _ => {
                                Err(Diagnostic::error(
                                    "Box index must evaluate to a number",
                                    index_expr.span,
                                ))
                            }
                        }
                    }

                    Value::Bag(entries) => {
                        let NodeKind::Ident(name) = &selector.kind else {
                            return Err(Diagnostic::error(
                                "Bag traversal requires a named selector",
                                selector.span,
                            ));
                        };

                        Ok(Value::Flag(
                            entries.contains_key(name),
                        ))
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
                    node.span,
                ),
            ),
        }
    }

    fn eval_loop_node_ctrl(
        &mut self,
        node: &Node,
    ) -> Result<Control, Diagnostic> {
        match &node.kind {
            NodeKind::Local(inner) => {
                match &inner.kind {
                    NodeKind::Define(def) => {
                        let value = self.eval_value(&def.value)?;
                        self.env
                            .define(def.name.clone(), value)
                            .map_err(|error| {
                                match error {
                                    DefineError::AlreadyDefined(name) => {
                                        Diagnostic::error(
                                            format!("identifier `{name}` is already defined"),
                                            node.span,
                                        )
                                    }
                                }
                            })?;
                    }

                    NodeKind::DefineEmpty(def) => {
                        self.env
                            .define(def.name.clone(), Value::Void)
                            .map_err(|error| {
                                match error {
                                    DefineError::AlreadyDefined(name) => {
                                        Diagnostic::error(
                                            format!("identifier `{name}` is already defined"),
                                            node.span,
                                        )
                                    }
                                }
                            })?;
                    }

                    NodeKind::Copy(copy) => {
                        self.env
                            .copy(copy.name.clone(), &copy.target)
                            .map_err(|error| {
                                match error {
                                    CopyError::UndefinedName(name) => {
                                        Diagnostic::error(
                                            format!("undeclared identifier `{name}`"),
                                            inner.span,
                                        )
                                    }

                                    CopyError::AlreadyDefined(name) => {
                                        Diagnostic::error(
                                            format!("identifier `{name}` is already defined"),
                                            inner.span,
                                        )
                                    }
                                }
                            })?;
                    }

                    NodeKind::Bind(bind) => {
                        self.env
                            .bind(
                                bind.name.clone(),
                                &bind.target,
                            )
                            .map_err(|error| match error {
                                BindError::UndefinedName(name) => {
                                    Diagnostic::error(
                                        format!(
                                            "undeclared identifier `{name}`",
                                        ),
                                        node.span,
                                    )
                                }

                                BindError::AlreadyDefined(name) => {
                                    Diagnostic::error(
                                        format!(
                                            "identifier `{name}` is already defined",
                                        ),
                                        node.span,
                                    )
                                }
                            })?;
                    }


                    NodeKind::Guard(guard) => {
                        let mut result = Value::Void;

                        for branch in &guard.branches {
                            let value = self.eval_value(&branch.expr)?;

                            if truth_of(&value) == Truth::True {
                                result = value;
                                break;
                            }
                        }

                        self.env
                            .define(guard.target.clone(), result)
                            .map_err(|error| {
                                match error {
                                    DefineError::AlreadyDefined(name) => {
                                        Diagnostic::error(
                                            format!("identifier `{name}` is already defined"),
                                            node.span,
                                        )
                                    }
                                }
                            })?;
                    }

                    _ => {
                        return Err(
                            Diagnostic::error(
                                "`loc` cannot modify this loop statement",
                                node.span,
                            ),
                        );
                    }
                }

                Ok(Control::Continue)
            }

             NodeKind::Mutate(mutate) => {
                let value = self.eval_value(&mutate.value)?;

                self.env
                    .assign(&mutate.name, value)
                    .map_err(|_| {
                        Diagnostic::error(
                            format!(
                                "undeclared identifier `{}`",
                                mutate.name,
                            ),
                            node.span,
                        )
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::Define(def) => {
                let value = self.eval_value(&def.value)?;

                self.env
                    .define(def.name.clone(), value)
                    .map_err(|error| {
                        match error {
                            DefineError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::DefineEmpty(def) => {
                self.env
                    .define(def.name.clone(), Value::Void)
                    .map_err(|error| {
                        match error {
                            DefineError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::Copy(copy) => {
                self.env
                    .copy(copy.name.clone(), &copy.target)
                    .map_err(|error| {
                        match error {
                            CopyError::UndefinedName(name) => {
                                Diagnostic::error(
                                    format!("undeclared identifier `{name}`"),
                                    node.span,
                                )
                            }

                            CopyError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::Bind(bind) => {
                self.env
                    .bind(
                        bind.name.clone(),
                        &bind.target,
                    )
                    .map_err(|error| match error {
                        BindError::UndefinedName(name) => {
                            Diagnostic::error(
                                format!(
                                    "undeclared identifier `{name}`",
                                ),
                                node.span,
                            )
                        }

                        BindError::AlreadyDefined(name) => {
                            Diagnostic::error(
                                format!(
                                    "identifier `{name}` is already defined",
                                ),
                                node.span,
                            )
                        }
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::Guard(guard) => {
                let mut result = Value::Void;

                for branch in &guard.branches {
                    let value = self.eval_value(&branch.expr)?;

                    if truth_of(&value) == Truth::True {
                        result = value;
                        break;
                    }
                }

                self.env
                    .define(guard.target.clone(), result)
                    .map_err(|error| {
                        match error {
                            DefineError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            _ => self.eval_node_ctrl(node),
        }
    }

    fn eval_node_ctrl(
        &mut self,
        node: &Node,
    ) -> Result<Control, Diagnostic> {
        match &node.kind {

            NodeKind::Define(def) => {
                let value = self.eval_value(&def.value)?;

                self.env
                    .define(def.name.clone(), value)
                    .map_err(|error| {
                        match error {
                            DefineError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::DefineEmpty(def) => {
                self.env
                    .define(def.name.clone(), Value::Void)
                    .map_err(|error| {
                        match error {
                            DefineError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::Copy(copy) => {
                self.env
                    .copy(copy.name.clone(), &copy.target)
                    .map_err(|error| {
                        match error {
                            CopyError::UndefinedName(name) => {
                                Diagnostic::error(
                                    format!("undeclared identifier `{name}`"),
                                    node.span,
                                )
                            }

                            CopyError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::Bind(bind) => {
                self.env
                    .bind(bind.name.clone(), &bind.target)
                    .map_err(|_| {
                        Diagnostic::error(
                            format!(
                                "undeclared identifier `{}`",
                                bind.target,
                            ),
                            node.span,
                        )
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::Mutate(mutate) => {
                let value = self.eval_value(&mutate.value)?;

                self.env
                    .assign(&mutate.name, value)
                    .map_err(|error| {
                        match error {
                            EnvError::UndefinedName(name) => {
                                Diagnostic::error(
                                    format!("undeclared identifier `{name}`"),
                                    node.span,
                                )
                            }

                            EnvError::StoneBinding(name) => {
                                Diagnostic::error(
                                    format!("cannot mutate stone binding `{name}`"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::Global(inner) => {
                match &inner.kind {
                    NodeKind::Mutate(mutate) => {
                        let value = self.eval_value(&mutate.value)?;

                        self.env
                            .assign_global(&mutate.name, value)
                            .map_err(|error| {
                                match error {
                                    EnvError::UndefinedName(name) => {
                                        Diagnostic::error(
                                            format!("undeclared identifier `{name}`"),
                                            inner.span,
                                        )
                                    }

                                    EnvError::StoneBinding(name) => {
                                        Diagnostic::error(
                                            format!("cannot mutate stone binding `{name}`"),
                                            inner.span,
                                        )
                                    }
                                }
                            })?;

                        Ok(Control::Continue)
                    }

                    NodeKind::Define(def) => {
                        let value = self.eval_value(&def.value)?;

                        self.env
                            .define_global(def.name.clone(), value)
                            .map_err(|error| match error {
                                DefineError::AlreadyDefined(name) => {
                                    Diagnostic::error(
                                        format!("identifier `{name}` is already defined"),
                                        inner.span,
                                    )
                                }
                            })?;

                        Ok(Control::Continue)
                    }

                    NodeKind::DefineEmpty(def) => {
                        self.env
                            .define_global(def.name.clone(), Value::Void)
                            .map_err(|error| match error {
                                DefineError::AlreadyDefined(name) => {
                                    Diagnostic::error(
                                        format!("identifier `{name}` is already defined"),
                                        inner.span,
                                    )
                                }
                            })?;

                        Ok(Control::Continue)
                    }

                    NodeKind::Copy(copy) => {
                        self.env
                            .copy_global(copy.name.clone(), &copy.target)
                            .map_err(|error| {
                                match error {
                                    GlobalCopyError::UndefinedName(name) => {
                                        Diagnostic::error(
                                            format!("undeclared identifier `{name}`"),
                                            inner.span,
                                        )
                                    }

                                    GlobalCopyError::AlreadyDefined(name) => {
                                        Diagnostic::error(
                                            format!("identifier `{name}` is already defined"),
                                            inner.span,
                                        )
                                    }
                                }
                            })?;

                        Ok(Control::Continue)
                    }

                    NodeKind::Bind(bind) => {
                        self.env
                            .bind_global(bind.name.clone(), &bind.target)
                            .map_err(|error| {
                                match error {
                                    GlobalBindError::UndefinedName(name) => {
                                        Diagnostic::error(
                                            format!("undeclared identifier `{name}`"),
                                            inner.span,
                                        )
                                    }

                                    GlobalBindError::LocalIdentity(name) => {
                                        Diagnostic::error(
                                            format!(
                                                "cannot create global bind to local identity `{name}`"
                                            ),
                                            inner.span,
                                        )
                                    }

                                    GlobalBindError::AlreadyDefined(name) => {
                                        Diagnostic::error(
                                            format!("identifier `{name}` is already defined"),
                                            inner.span,
                                        )
                                    }
                                }
                            })?;

                        Ok(Control::Continue)
                    }
                    
                    NodeKind::Guard(guard) => {
                        let mut result = Value::Void;

                        for branch in &guard.branches {
                            let value = self.eval_value(&branch.expr)?;

                            if truth_of(&value) == Truth::True {
                                result = value;
                                break;
                            }
                        }

                        self.env
                            .define_global(guard.target.clone(), result)
                            .map_err(|error| match error {
                                DefineError::AlreadyDefined(name) => {
                                    Diagnostic::error(
                                        format!("identifier `{name}` is already defined"),
                                        inner.span,
                                    )
                                }
                            })?;

                        Ok(Control::Continue)
                    }

                    NodeKind::Func(func) => {
                        let value = Value::Func(
                            crate::compiler::semantics::value::Func {
                                name: func.name.clone(),
                                params: func.params.clone(),
                                body: func.body.clone(),
                            },
                        );

                        self.env
                            .define_global(func.name.clone(), value)
                            .map_err(|error| match error {
                                DefineError::AlreadyDefined(name) => {
                                    Diagnostic::error(
                                        format!("identifier `{name}` is already defined"),
                                        inner.span,
                                    )
                                }
                            })?;

                        Ok(Control::Continue)
                    }                    

                    _ => Err(
                        Diagnostic::error(
                            "global modifier is not implemented for this statement yet",
                            node.span,
                        ),
                    ),
                }
            }

            NodeKind::Stone(inner) => {
                let target = match &inner.kind {
                    NodeKind::Define(def) => def.name.as_str(),
                    NodeKind::DefineEmpty(def) => def.name.as_str(),
                    NodeKind::Copy(copy) => copy.name.as_str(),
                    NodeKind::Bind(bind) => bind.name.as_str(),
                    NodeKind::Guard(guard) => guard.target.as_str(),
                    NodeKind::Mutate(mutate) => mutate.name.as_str(),

                    NodeKind::Local(_) => {
                        return Err(
                            Diagnostic::error(
                                "combined stone scope modifiers are not implemented yet",
                                node.span,
                            ),
                        );
                    }

                    NodeKind::Global(global_inner) => {
                        match &global_inner.kind {
                            NodeKind::Mutate(mutate) => {
                                let value = self.eval_value(&mutate.value)?;

                                self.env
                                    .assign_global(&mutate.name, value)
                                    .map_err(|error| {
                                        match error {
                                            EnvError::UndefinedName(name) => {
                                                Diagnostic::error(
                                                    format!("undeclared identifier `{name}`"),
                                                    global_inner.span,
                                                )
                                            }

                                            EnvError::StoneBinding(name) => {
                                                Diagnostic::error(
                                                    format!("cannot mutate stone binding `{name}`"),
                                                    global_inner.span,
                                                )
                                            }
                                        }
                                    })?;

                                self.env
                                    .mark_global_stone(&mutate.name)
                                    .map_err(|error| {
                                        match error {
                                            EnvError::UndefinedName(name) => {
                                                Diagnostic::error(
                                                    format!("undeclared identifier `{name}`"),
                                                    global_inner.span,
                                                )
                                            }

                                            EnvError::StoneBinding(name) => {
                                                Diagnostic::error(
                                                    format!("binding `{name}` is already stone"),
                                                    global_inner.span,
                                                )
                                            }
                                        }
                                    })?;

                                return Ok(Control::Continue);
                            }

                            NodeKind::Define(def) => {
                                let value = self.eval_value(&def.value)?;

                                self.env
                                    .define_global(def.name.clone(), value)
                                    .map_err(|error| match error {
                                        DefineError::AlreadyDefined(name) => {
                                            Diagnostic::error(
                                                format!("identifier `{name}` is already defined"),
                                                inner.span,
                                            )
                                        }
                                    })?;

                                self.env
                                    .mark_global_stone(&def.name)
                                    .map_err(|error| {
                                        match error {
                                            EnvError::UndefinedName(name) => {
                                                Diagnostic::error(
                                                    format!("undeclared identifier `{name}`"),
                                                    global_inner.span,
                                                )
                                            }

                                            EnvError::StoneBinding(name) => {
                                                Diagnostic::error(
                                                    format!("binding `{name}` is already stone"),
                                                    global_inner.span,
                                                )
                                            }
                                        }
                                    })?;

                                return Ok(Control::Continue);
                            }

                            NodeKind::DefineEmpty(def) => {
                                self.env
                                    .define_global(def.name.clone(), Value::Void)
                                    .map_err(|error| match error {
                                        DefineError::AlreadyDefined(name) => {
                                            Diagnostic::error(
                                                format!("identifier `{name}` is already defined"),
                                                inner.span,
                                            )
                                        }
                                    })?;

                                self.env
                                    .mark_global_stone(&def.name)
                                    .map_err(|error| {
                                        match error {
                                            EnvError::UndefinedName(name) => {
                                                Diagnostic::error(
                                                    format!("undeclared identifier `{name}`"),
                                                    global_inner.span,
                                                )
                                            }

                                            EnvError::StoneBinding(name) => {
                                                Diagnostic::error(
                                                    format!("binding `{name}` is already stone"),
                                                    global_inner.span,
                                                )
                                            }
                                        }
                                    })?;

                                return Ok(Control::Continue);
                            }
                            
                            NodeKind::Copy(copy) => {
                                self.env
                                    .copy_global(copy.name.clone(), &copy.target)
                                    .map_err(|error| {
                                    match error {
                                        GlobalCopyError::UndefinedName(name) => {
                                            Diagnostic::error(
                                                format!("undeclared identifier `{name}`"),
                                                inner.span,
                                            )
                                        }

                                        GlobalCopyError::AlreadyDefined(name) => {
                                            Diagnostic::error(
                                                format!("identifier `{name}` is already defined"),
                                                inner.span,
                                            )
                                        }
                                    }
                                })?;

                                self.env
                                    .mark_global_stone(&copy.name)
                                    .map_err(|error| {
                                        match error {
                                            EnvError::UndefinedName(name) => {
                                                Diagnostic::error(
                                                    format!("undeclared identifier `{name}`"),
                                                    global_inner.span,
                                                )
                                            }

                                            EnvError::StoneBinding(name) => {
                                                Diagnostic::error(
                                                    format!("binding `{name}` is already stone"),
                                                    global_inner.span,
                                                )
                                            }
                                        }
                                    })?;

                                return Ok(Control::Continue);
                            }

                            NodeKind::Bind(bind) => {
                                self.env
                                    .bind_global(bind.name.clone(), &bind.target)
                                    .map_err(|error| {
                                        match error {
                                            GlobalBindError::UndefinedName(name) => {
                                                Diagnostic::error(
                                                    format!("undeclared identifier `{name}`"),
                                                    global_inner.span,
                                                )
                                            }

                                            GlobalBindError::LocalIdentity(name) => {
                                                Diagnostic::error(
                                                    format!(
                                                        "cannot create global bind to local identity `{name}`"
                                                    ),
                                                    global_inner.span,
                                                )
                                            }

                                            GlobalBindError::AlreadyDefined(name) => {
                                                Diagnostic::error(
                                                    format!("identifier `{name}` is already defined"),
                                                    inner.span,
                                                )
                                            }
                                        }
                                    })?;

                                self.env
                                    .mark_global_stone(&bind.name)
                                    .map_err(|error| {
                                        match error {
                                            EnvError::UndefinedName(name) => {
                                                Diagnostic::error(
                                                    format!("undeclared identifier `{name}`"),
                                                    global_inner.span,
                                                )
                                            }

                                            EnvError::StoneBinding(name) => {
                                                Diagnostic::error(
                                                    format!("binding `{name}` is already stone"),
                                                    global_inner.span,
                                                )
                                            }
                                        }
                                    })?;

                                return Ok(Control::Continue);
                            }
                            
                            NodeKind::Guard(guard) => {
                                let mut result = Value::Void;

                                for branch in &guard.branches {
                                    let value = self.eval_value(&branch.expr)?;

                                    if truth_of(&value) == Truth::True {
                                        result = value;
                                        break;
                                    }
                                }

                                self.env
                                    .define_global(guard.target.clone(), result)
                                    .map_err(|error| match error {
                                        DefineError::AlreadyDefined(name) => {
                                            Diagnostic::error(
                                                format!("identifier `{name}` is already defined"),
                                                inner.span,
                                            )
                                        }
                                    })?;

                                self.env
                                    .mark_global_stone(&guard.target)
                                    .map_err(|error| {
                                        match error {
                                            EnvError::UndefinedName(name) => {
                                                Diagnostic::error(
                                                    format!("undeclared identifier `{name}`"),
                                                    global_inner.span,
                                                )
                                            }

                                            EnvError::StoneBinding(name) => {
                                                Diagnostic::error(
                                                    format!("binding `{name}` is already stone"),
                                                    global_inner.span,
                                                )
                                            }
                                        }
                                    })?;

                                return Ok(Control::Continue);
                            }

                            _ => {
                                return Err(
                                    Diagnostic::error(
                                        "combined stone scope modifiers are not implemented yet",
                                        node.span,
                                    ),
                                );
                            }
                        }
                    }

                    _ => {
                        return Err(
                            Diagnostic::error(
                                "`stone` cannot modify this statement",
                                node.span,
                            ),
                        );
                    }
                }
                .to_string();

                // The inner operation happens first.
                let control = self.eval_node_ctrl(inner)?;

                // Only stone the identity after successful completion.
                self.env
                    .mark_stone(&target)
                    .map_err(|error| {
                        match error {
                            EnvError::UndefinedName(name) => {
                                Diagnostic::error(
                                    format!("undeclared identifier `{name}`"),
                                    node.span,
                                )
                            }

                            EnvError::StoneBinding(name) => {
                                Diagnostic::error(
                                    format!("binding `{name}` is already stone"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(control)
            }

            NodeKind::Guard(guard) => {
                let mut result = Value::Void;

                for branch in &guard.branches {
                    let value = self.eval_value(&branch.expr)?;

                    if truth_of(&value) == Truth::True {
                        result = value;
                        break;
                    }
                }

                self.env
                    .define(guard.target.clone(), result)
                    .map_err(|error| {
                        match error {
                            DefineError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            NodeKind::Ret(ret) => {
                let value = match &ret.value {
                    Some(value_node) => self.eval_value(value_node)?,
                    None => Value::Void,
                };

                Ok(Control::Return(value))
            }

            NodeKind::Loop(loop_node) => {
                self.env.push_scope();

                let result = (|| {
                    // Setup executes exactly once.
                    for setup_node in &loop_node.setup {
                        match self.eval_loop_node_ctrl(setup_node)? {
                            Control::Continue => {}

                            Control::Return(value) => {
                                return Ok(Control::Return(value));
                            }
                        }
                    }

                    // The same loop scope remains active for every iteration.
                    loop {
                        let condition = self.eval_value(
                            loop_node.condition.as_ref(),
                        )?;

                        if truth_of(&condition) == Truth::False {
                            break;
                        }

                        for process_node in &loop_node.process {
                            match self.eval_loop_node_ctrl(process_node)? {
                                Control::Continue => {}

                                Control::Return(value) => {
                                    return Ok(Control::Return(value));
                                }
                            }
                        }
                    }

                    Ok(Control::Continue)
                })();

                self.env.pop_scope();

                result
            }

            NodeKind::Block(block) => {
                self.env.push_scope();

                let result = (|| {
                    for segment in &block.segments {
                        let mut local_names = Vec::new();

                        let segment_result: Result<Control, Diagnostic> = (|| {
                            for segment_node in &segment.nodes {
                                let control = match &segment_node.kind {
                                    NodeKind::Stone(stone_inner) => {
                                        if let NodeKind::Local(local_inner) = &stone_inner.kind {
                                            if let NodeKind::Mutate(mutate) = &local_inner.kind {
                                                // Evaluate before localization so the RHS sees the
                                                // currently visible identity.
                                                let value = self.eval_value(&mutate.value)?;

                                                let localized_names = self
                                                    .env
                                                    .localize_identity(&mutate.name, value)
                                                    .map_err(|_| {
                                                        Diagnostic::error(
                                                            format!(
                                                                "undeclared identifier `{}`",
                                                                mutate.name,
                                                            ),
                                                            local_inner.span,
                                                        )
                                                    })?;

                                                // Register these before any later operation so normal
                                                // segment cleanup restores the outer identity.
                                                local_names.extend(localized_names);

                                                // The mutation has already happened during localization.
                                                // Now stone only the localized shared identity.
                                                self.env
                                                    .mark_stone(&mutate.name)
                                                    .map_err(|error| {
                                                        match error {
                                                            EnvError::UndefinedName(name) => {
                                                                Diagnostic::error(
                                                                    format!(
                                                                        "undeclared identifier `{name}`"
                                                                    ),
                                                                    local_inner.span,
                                                                )
                                                            }

                                                            EnvError::StoneBinding(name) => {
                                                                Diagnostic::error(
                                                                    format!(
                                                                        "binding `{name}` is already stone"
                                                                    ),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                continue;
                                            }

                                            if let NodeKind::Define(def) = &local_inner.kind {
                                                if self.env.lookup(&def.name).is_some() {
                                                    return Err(
                                                        Diagnostic::error(
                                                            format!(
                                                                "identifier `{}` is already defined",
                                                                def.name
                                                            ),
                                                            local_inner.span,
                                                        ),
                                                    );
                                                }

                                                let value = self.eval_value(&def.value)?;

                                                local_names.push((
                                                    def.name.clone(),
                                                    None,
                                                ));

                                                self.env
                                                    .define(def.name.clone(), value)
                                                    .map_err(|error| {
                                                        match error {
                                                            DefineError::AlreadyDefined(name) => {
                                                                Diagnostic::error(
                                                                    format!("identifier `{name}` is already defined"),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                self.env
                                                    .mark_stone(&def.name)
                                                    .map_err(|error| {
                                                        match error {
                                                            EnvError::UndefinedName(name) => {
                                                                Diagnostic::error(
                                                                    format!("undeclared identifier `{name}`"),
                                                                    local_inner.span,
                                                                )
                                                            }

                                                            EnvError::StoneBinding(name) => {
                                                                Diagnostic::error(
                                                                    format!("binding `{name}` is already stone"),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                continue;
                                            }

                                            if let NodeKind::DefineEmpty(def) = &local_inner.kind {
                                                if self.env.lookup(&def.name).is_some() {
                                                    return Err(
                                                        Diagnostic::error(
                                                            format!(
                                                                "identifier `{}` is already defined",
                                                                def.name
                                                            ),
                                                            local_inner.span,
                                                        ),
                                                    );
                                                }

                                                local_names.push((
                                                    def.name.clone(),
                                                    None,
                                                ));

                                                self.env
                                                    .define(def.name.clone(), Value::Void)
                                                    .map_err(|error| {
                                                        match error {
                                                            DefineError::AlreadyDefined(name) => {
                                                                Diagnostic::error(
                                                                    format!("identifier `{name}` is already defined"),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                self.env
                                                    .mark_stone(&def.name)
                                                    .map_err(|error| {
                                                        match error {
                                                            EnvError::UndefinedName(name) => {
                                                                Diagnostic::error(
                                                                    format!("undeclared identifier `{name}`"),
                                                                    local_inner.span,
                                                                )
                                                            }

                                                            EnvError::StoneBinding(name) => {
                                                                Diagnostic::error(
                                                                    format!("binding `{name}` is already stone"),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                continue;
                                            }
                                            
                                            if let NodeKind::Copy(copy) = &local_inner.kind {
                                                if self.env.lookup(&copy.name).is_some() {
                                                    return Err(
                                                        Diagnostic::error(
                                                            format!(
                                                                "identifier `{}` is already defined",
                                                                copy.name
                                                            ),
                                                            local_inner.span,
                                                        ),
                                                    );
                                                }

                                                local_names.push((
                                                    copy.name.clone(),
                                                    None,
                                                ));

                                                self.env
                                                    .copy(copy.name.clone(), &copy.target)
                                                    .map_err(|error| {
                                                        match error {
                                                            CopyError::UndefinedName(name) => {
                                                                Diagnostic::error(
                                                                    format!("undeclared identifier `{name}`"),
                                                                    local_inner.span,
                                                                )
                                                            }

                                                            CopyError::AlreadyDefined(name) => {
                                                                Diagnostic::error(
                                                                    format!("identifier `{name}` is already defined"),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                self.env
                                                    .mark_stone(&copy.name)
                                                    .map_err(|error| {
                                                        match error {
                                                            EnvError::UndefinedName(name) => {
                                                                Diagnostic::error(
                                                                    format!("undeclared identifier `{name}`"),
                                                                    local_inner.span,
                                                                )
                                                            }

                                                            EnvError::StoneBinding(name) => {
                                                                Diagnostic::error(
                                                                    format!("binding `{name}` is already stone"),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                continue;
                                            }
                                            
                                            if let NodeKind::Bind(bind) = &local_inner.kind {
                                                if self.env.lookup(&bind.name).is_some() {
                                                    return Err(
                                                        Diagnostic::error(
                                                            format!(
                                                                "identifier `{}` is already defined",
                                                                bind.name
                                                            ),
                                                            local_inner.span,
                                                        ),
                                                    );
                                                }

                                                let target_slot = self
                                                    .env
                                                    .lookup(&bind.target)
                                                    .ok_or_else(|| {
                                                        Diagnostic::error(
                                                            format!(
                                                                "undeclared identifier `{}`",
                                                                bind.target
                                                            ),
                                                            local_inner.span,
                                                        )
                                                    })?;

                                                let value = target_slot.borrow().value.clone();

                                                // First create the local alias to the existing identity.
                                                local_names.push((
                                                    bind.name.clone(),
                                                    None,
                                                ));

                                                self.env
                                                    .bind(bind.name.clone(), &bind.target)
                                                    .map_err(|_| {
                                                        Diagnostic::error(
                                                            format!(
                                                                "undeclared identifier `{}`",
                                                                bind.target
                                                            ),
                                                            local_inner.span,
                                                        )
                                                    })?;

                                                // Then localize the entire shared identity. This makes every
                                                // name referring to that identity inside this local lifetime
                                                // point at the localized slot, while preserving the outer slot.
                                                let localized_names = self
                                                    .env
                                                    .localize_identity(&bind.name, value)
                                                    .map_err(|_| {
                                                        Diagnostic::error(
                                                            format!(
                                                                "undeclared identifier `{}`",
                                                                bind.name
                                                            ),
                                                            local_inner.span,
                                                        )
                                                    })?;

                                                local_names.extend(localized_names);

                                                // Stone only the localized shared identity.
                                                self.env
                                                    .mark_stone(&bind.name)
                                                    .map_err(|error| {
                                                        match error {
                                                            EnvError::UndefinedName(name) => {
                                                                Diagnostic::error(
                                                                    format!("undeclared identifier `{name}`"),
                                                                    local_inner.span,
                                                                )
                                                            }

                                                            EnvError::StoneBinding(name) => {
                                                                Diagnostic::error(
                                                                    format!("binding `{name}` is already stone"),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                continue;
                                            }
                                            
                                            if let NodeKind::Guard(guard) = &local_inner.kind {
                                                if self.env.lookup(&guard.target).is_some() {
                                                    return Err(
                                                        Diagnostic::error(
                                                            format!(
                                                                "identifier `{}` is already defined",
                                                                guard.target
                                                            ),
                                                            local_inner.span,
                                                        ),
                                                    );
                                                }

                                                let mut result = Value::Void;

                                                for branch in &guard.branches {
                                                    let value = self.eval_value(&branch.expr)?;

                                                    if truth_of(&value) == Truth::True {
                                                        result = value;
                                                        break;
                                                    }
                                                }

                                                local_names.push((
                                                    guard.target.clone(),
                                                    None,
                                                ));

                                                self.env
                                                    .define(guard.target.clone(), result)
                                                    .map_err(|error| {
                                                        match error {
                                                            DefineError::AlreadyDefined(name) => {
                                                                Diagnostic::error(
                                                                    format!("identifier `{name}` is already defined"),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                self.env
                                                    .mark_stone(&guard.target)
                                                    .map_err(|error| {
                                                        match error {
                                                            EnvError::UndefinedName(name) => {
                                                                Diagnostic::error(
                                                                    format!("undeclared identifier `{name}`"),
                                                                    local_inner.span,
                                                                )
                                                            }

                                                            EnvError::StoneBinding(name) => {
                                                                Diagnostic::error(
                                                                    format!("binding `{name}` is already stone"),
                                                                    local_inner.span,
                                                                )
                                                            }
                                                        }
                                                    })?;

                                                continue;
                                            }                                            
                                        }

                                        self.eval_node_ctrl(segment_node)?
                                    }
                                    NodeKind::Local(inner) => {
                                        if let NodeKind::Mutate(mutate) = &inner.kind {
                                            // Evaluate before localization so the RHS sees the
                                            // currently visible identity.
                                            let value = self.eval_value(&mutate.value)?;

                                            let localized_names = self
                                                .env
                                                .localize_identity(&mutate.name, value)
                                                .map_err(|_| {
                                                    Diagnostic::error(
                                                        format!(
                                                            "undeclared identifier `{}`",
                                                            mutate.name,
                                                        ),
                                                        inner.span,
                                                    )
                                                })?;

                                            local_names.extend(localized_names);

                                            continue;
                                        }
                                        if let Some(name) = local_name(inner) {
                                            if self.env.lookup(name).is_some() {
                                                return Err(
                                                    Diagnostic::error(
                                                        format!(
                                                            "identifier `{name}` is already defined"
                                                        ),
                                                        inner.span,
                                                    ),
                                                );
                                            }

                                            local_names.push((
                                                name.to_string(),
                                                None,
                                            ));
                                        }

                                        self.eval_node_ctrl(inner)?
                                    }
                                    _ => self.eval_node_ctrl(segment_node)?,
                                };

                                if let Control::Return(value) = control {
                                    return Ok(Control::Return(value));
                                }
                            }

                            Ok(Control::Continue)
                        })();

                        for (name, previous) in local_names.into_iter().rev() {
                            self.env.remove_current(&name);

                            if let Some(slot) = previous {
                                self.env.restore_current(name, slot);
                            }
                        }

                        if let Control::Return(value) = segment_result? {
                            return Ok(Control::Return(value));
                        }
                    }

                    Ok(Control::Continue)
                })();

                self.env.pop_scope();

                result
            }

            NodeKind::Func(func) => {
                let value = Value::Func(
                    crate::compiler::semantics::value::Func {
                        name: func.name.clone(),
                        params: func.params.clone(),
                        body: func.body.clone(),
                    },
                );

                self.env
                    .define(func.name.clone(), value)
                    .map_err(|error| {
                        match error {
                            DefineError::AlreadyDefined(name) => {
                                Diagnostic::error(
                                    format!("identifier `{name}` is already defined"),
                                    node.span,
                                )
                            }
                        }
                    })?;

                Ok(Control::Continue)
            }

            _ => {
                let _ = self.eval_value(node)?;

                Ok(Control::Continue)
            }
        }
    }
}
