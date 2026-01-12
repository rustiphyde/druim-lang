#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // ===== Identifiers & literals =====
    Ident,

    NumLit,   // whole number literal
    DecLit,   // decimal literal
    TextLit,  // quoted text

    // ===== Keywords (types) =====
    KwNum,
    KwDec,
    KwFlag,
    KwText,
    KwEmp,

    // ===== Assignment & binding =====
    QAssign,       // ?=
    Bind,          // :=

    // ===== Colon family =====
    Colon,         // :
    Scope,         // ::
    Present,       // :?
    Cast,          // :>

    // ===== Arithmetic =====
    Add,           // +
    Sub,           // -
    Mul,           // *
    Div,           // /
    Mod,           // %

    // ===== Comparison =====
    Eq,            // ==
    Ne,            // !=
    Lt,            // <
    Le,            // <=
    Gt,            // >
    Ge,            // >=

    // ===== Logical =====
    And,           // &&
    Or,            // ||
    Not,           // !

    // ===== Flow =====
    Pipe,          // |>
    ArrowR,        // ->
    ArrowL,        // <-

    // ===== Define =====
    Define,        // =
    DefineEmpty,   // =;

    // ===== Punctuation =====
    LParen,        // (
    RParen,        // )
    Comma,         // ,
    Semicolon,     // ;

    // ===== Blocks =====
    BlockStmtStart, // :{
    BlockStmtEnd,   // }:
    BlockExprStart, // :[
    BlockExprEnd,   // ]:
    BlockFuncStart, // :(
    BlockFuncEnd,   // ):


    // ===== Special =====
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub pos: usize, // byte offset in source
}
