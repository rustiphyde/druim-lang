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
    KwVoid,

    // ===== Keywords (expressions) =====
    KwFn,   // fn
    KwRet,  // ret
    KwLoc, // loc 

    // ===== Assignment & binding =====


    // ===== Colon family =====
    Colon,         // :
    Has,           // ::
    Present,       // :?
    Bind,          // :>

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

    // ===== Copy & Guard =====
    Copy,          // :=
    Guard,         // ?=

    // ===== Punctuation =====
    LParen,        // (
    RParen,        // )
    Comma,         // ,
    Semicolon,     // ;

    // ===== Blocks =====
    BlockStart, // :{
    BlockEnd,   // }:
    BlockChain, // }{
    ArrayStart, // :[
    ArrayEnd,   // ]:
    ArrayChain, // ][
    FuncStart, // :(
    FuncEnd,   // ):
    FuncChain, // )(
    // ===== Special =====
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub pos: usize, // byte offset in source
}
