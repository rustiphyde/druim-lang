#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer number.
    Num(i64),

    /// Decimal number (kept as text to preserve precision).
    Dec(String),

    /// Boolean value.
    Flag(bool),

    /// Text value.
    Text(String),

    /// Explicit absence of value.
    ///
    /// `void` always evaluates to a false flag.
    /// There is no `undefined` in Druim.
    Void,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // ===== Atoms =====
    Ident(String),
    Lit(Literal),

    // ===== Unary =====
    Not(Box<Node>),
    Neg(Box<Node>),

    // ===== Arithmetic =====
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Mod(Box<Node>, Box<Node>),

    // ===== Comparison =====
    Eq(Box<Node>, Box<Node>),
    Ne(Box<Node>, Box<Node>),
    Lt(Box<Node>, Box<Node>),
    Le(Box<Node>, Box<Node>),
    Gt(Box<Node>, Box<Node>),
    Ge(Box<Node>, Box<Node>),

    // ===== Logical =====
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),

    Has(Box<Node>, Box<Node>),     // ::
    Present(Box<Node>, Box<Node>),   // :?

    // ===== Flow =====
    Pipe(Box<Node>, Box<Node>),      // |>
    Block(Block),
    Ret(Ret),
    Define(Define),
    DefineEmpty(DefineEmpty),
    Copy(Copy),
    Bind(Bind),
    Guard(Guard),
    Func(Func),
    Call(Call)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    pub name: String,
    pub params: Vec<Param>,
    pub bodies: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub callee: Box<Node>,
    pub args: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub default: Option<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ret {
    pub value: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Define {
    pub name: String,
    pub value: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefineEmpty {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Copy {
    pub name: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bind {
    pub name: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Guard {
    pub target: String,
    pub branches: Vec<Node>,
}






