use crate::compiler::ast::Literal;

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
