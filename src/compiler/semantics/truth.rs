use crate::compiler::semantics::value::Value;

/// Result of evaluating truth in Druim.
///
/// Truth is always explicit and resolves to a `flag`.
/// There is no implicit truthiness and no `undefined`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Truth {
    True,
    False,
}

impl Truth {
    pub fn as_bool(self) -> bool {
        matches!(self, Truth::True)
    }
}

/// Determine truth value of a runtime `Value`.
///
/// RULES (LOCKED):
///
/// - `flag(true)`  → true
/// - `flag(false)` → false
/// - `emp`         → false
/// - `num(0)`      → false
/// - `num(!0)`     → true
/// - `dec(0.0)`    → false
/// - `dec(!0.0)`   → true
/// - `text("")`    → false
/// - `text(any)`   → true
///
/// Any future value kinds MUST be handled explicitly.
pub fn truth_of(value: &Value) -> Truth {
    match value {
        Value::Flag(b) => {
            if *b { Truth::True } else { Truth::False }
        }

        Value::Emp => Truth::False,

        Value::Num(n) => {
            if *n == 0 { Truth::False } else { Truth::True }
        }

        Value::Dec(d) => {
            // Decimals are stored as text.
            // Semantic rule: zero parses to false, any non-zero parses to true.
            match d.parse::<f64>() {
                Ok(v) if v != 0.0 => Truth::True,
                _ => Truth::False,
            }
        }

        Value::Text(t) => {
            if t.is_empty() { Truth::False } else { Truth::True }
        }
    }
}
