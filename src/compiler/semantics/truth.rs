use super::Value;

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Flag(b) => *b,
        Value::Num(n) => *n != 0,
        Value::Dec(s) => s != "0" && s != "0.0",
        Value::Text(s) => !s.is_empty(),
        Value::Emp => false,
    }
}
