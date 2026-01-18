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
    /// `emp` always evaluates to a false flag.
    /// There is no `undefined` in Druim.
    Void,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // ===== Atoms =====
    Ident(String),
    Lit(Literal),

    // ===== Unary =====
    Not(Box<Expr>),
    Neg(Box<Expr>),

    // ===== Arithmetic =====
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),

    // ===== Comparison =====
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),

    // ===== Logical =====
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),

    Has(Box<Expr>, Box<Expr>),     // ::
    Present(Box<Expr>, Box<Expr>),   // :?
    Cast(Box<Expr>, Box<Expr>),      // :>

    // ===== Flow =====
    Pipe(Box<Expr>, Box<Expr>),      // |>

    // ===== Calls =====
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// Expression block.
    ///
    /// Evaluates the enclosed expression and yields its value.
    /// Establishes a computed region without introducing statements.
    ///
    /// Syntax:
    ///     :[ expr ][ expr]:
    BlockExpr {
        expr: Box<Expr>,
    },

    /// Named function block.
    ///
    /// Introduced explicitly with the `fn` keyword and a snake_cased identifier.
    /// Establishes a function-local scope.
    /// Variables declared inside are visible only within the function
    /// and chained bodies unless restricted by future keywords.
    ///
    /// Syntax:
    ///     fn my_function :( args )( body1 )( body2 ):
    FnBlock {
        name: String,
        args: Vec<Expr>,
        bodies: Vec<Expr>,
    },
    
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Structured statement block.
    ///
    /// Represents a sequence of statements executed in order,
    /// enclosed by block delimiters. Blocks establish scope
    /// boundaries and do not require semicolon termination.
    ///
    /// Syntax:
    ///     :{ stmt* }{ stmt* }:
    Block {
        stmts: Vec<Stmt>,
    },
    /// Imperative data mutation.
    /// 
    /// Transfers the value produced by `source` into `target`.
    /// This is a state-changing operation and represents assignment
    /// in Druim. It does not produce a value and must be terminated
    /// with a semicolon.
    ///
    /// Syntax:
    ///     target <- source;
    AssignFrom {
        target: Expr,
        source: Expr,
    },

    /// Directional data emission.
    /// 
    /// Sends the value produced by `value` into `destination`.
    /// This represents outward flow or delivery of data rather than
    /// local mutation. It is a statement-only operation and does not
    /// produce a value.
    ///
    /// Syntax:
    ///     value -> destination;
    SendTo {
        value: Expr,
        destination: Expr,
    },

    /// Declarative name binding.
    /// 
    /// Defines a new identifier and binds it to the result of `value`.
    /// This operation establishes a definition, not a mutation.
    /// The left-hand side must be a single identifier, and the binding
    /// may be further constrained by modifiers such as `stone`.
    ///
    /// Syntax:
    ///     name = value;
    Define {
        name: String,
        value: Expr,
    },
    
    /// Declarative empty binding.
    ///
    /// Declares an identifier without assigning a value.
    /// This establishes the name in the current scope
    /// for later binding or mutation.
    ///
    /// Syntax:
    ///     name =;
    DefineEmpty {
        name: String,
    },
        /// Declarative binding to an existing identifier.
    ///
    /// Binds `name` to an already-defined identifier `target`.
    /// Does not create or compute a new value.
    ///
    /// Syntax:
    ///     name := target;
    Bind {
        name: String,
        target: String,
    },
     /// Guarded assignment.
    ///
    /// Evaluates `condition`.
    /// If truthy, assigns `target = value`.
    /// Otherwise continues to the next guard or resolves to `emp`.
    ///
    /// Syntax:
    ///     x ?= y;
    ///     x ?= y : z;
    ///     x ?= y : z : v;
    ///
    /// If no fallback branch succeeds, `target` is assigned `emp`.
    Guard {
        target: String,
        branches: Vec<Expr>,
    }



}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}


