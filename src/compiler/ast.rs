use crate::compiler::error::Span;

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
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // ===== Atoms =====
    Ident(String),
    Lit(Literal),

    // ===== Collections =====
    Box(BoxLiteral),
    Bag(BagLiteral),

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

    // ===== Traversal =====
    Get(Box<Node>, Box<Node>), // ::
    Has(Box<Node>, Box<Node>), // :?
    Index(Box<Node>),          // [expression]

    // ===== Flow =====
    Pipe(Box<Node>, Box<Node>),      // |>
    Block(Block),
    Local(Box<Node>),
    Ret(Ret),
    Define(Define),
    DefineEmpty(DefineEmpty),
    Copy(Copy),
    Bind(Bind),
    Guard(Guard),
    Func(Func),
    Call(Call),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxLiteral {
    pub values: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BagLiteral {
    pub entries: Vec<BagEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BagEntry {
    pub name: String,
    pub value: Node,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub callee: Box<Node>,
    pub args: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub segments: Vec<BlockSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockSegment {
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
    pub branches: Vec<GuardBranch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuardBranch {
    pub expr: Node
}






