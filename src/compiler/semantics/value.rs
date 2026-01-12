#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(i64),
    Dec(String),
    Flag(bool),
    Text(String),
    Emp,
}
