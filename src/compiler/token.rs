#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // ===== Identifiers & literals =====
    Ident,

    NumLit,   // whole number literal
    DecLit,   // decimal literal
    TextLit,  // quoted text
    FlagLit,  // true or false
    
    TextStart,
    TextEnd,
    InterpStart,
    InterpEnd,

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
    KwGlo, // glo
    KwStone, // stone 

    // ===== Colon family =====
    Colon,         // :
    Get,           // ::
    Has,           // :?
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

    // ===== Define =====
    Define,        // =
    DefineEmpty,   // =;

    // ===== Copy & Guard =====
    Copy,          // :=
    Guard,         // ?=

    // ===== Mutation =====
    Mutate,        // <<

    // ===== Output =====
    Print,         // |>

    // ===== Punctuation =====
    LParen,        // (
    RParen,        // )
    LBracket,      // [
    RBracket,      // ]
    Comma,         // ,
    Semicolon,     // ;

    // ===== Blocks =====
    BlockStart, // :{
    BlockEnd,   // }:
    BlockChain, // }{

    LoopStart, // :<
    LoopSplit, // >?<
    LoopEnd,   // >:

    BoxStart, // :[
    BoxEnd,   // ]:
    BagStart, // :|
    BagEnd,   // |:
    FuncStart, // :(
    FuncEnd,   // ):
    FuncChain, // )(

    // ===== Program boundary =====
    ProgramBoundary, // :-:-:

    // ===== Special =====
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub pos: usize, // byte offset in source
}
