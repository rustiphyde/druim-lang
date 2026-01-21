use crate::compiler::ast::{Literal, Expr};

/// Runtime value representation.
///
/// This is the evaluated form of expressions.
/// It is deliberately separate from the AST to:
/// - prevent syntax from leaking into semantics
/// - allow future optimization / VM layers
/// - make truth semantics explicit and testable
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Integer value.
    Num(i64),

    /// Decimal value (kept as text to preserve precision).
    Dec(String),

    /// Boolean value.
    Flag(bool),

    /// Text value.
    Text(String),

    /// Explicit absence of value.
    ///void`:
    /// - always exists
    /// - is never undefined
    /// - always evaluates to false
    Void,

        /// User-defined function value.
    ///
    /// Represents a callable function introduced by a `fn` block.
    /// Functions are first-class values and may be:
    /// - defined in the environment
    /// - passed as arguments
    /// - invoked via call expressions
    ///
    /// Function values do not carry execution state.
    /// A fresh function-local scope is created on each invocation.
    ///
    /// Return behavior:
    /// - `ret expr;` returns the evaluated expression
    /// - `ret;` returns `void`
    /// - If no `ret` executes, the function implicitly returns `void`
    Func(Function),
    
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub bodies: Vec<Expr>,
}


impl Value {
    /// Construct a runtime value from a literal.
    ///
    /// This performs no evaluation or coercion.
    /// Truth semantics are handled separately.
    pub fn from_literal(lit: &Literal) -> Self {
        match lit {
            Literal::Num(n) => Value::Num(*n),
            Literal::Dec(d) => Value::Dec(d.clone()),
            Literal::Flag(b) => Value::Flag(*b),
            Literal::Text(t) => Value::Text(t.clone()),
            Literal::Void => Value::Void,
        }
    }
}
