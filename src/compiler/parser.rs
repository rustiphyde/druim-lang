use crate::compiler::ast::{Node, Literal, Program, Param, Define, DefineEmpty, Copy, Bind, Guard, Block, Ret, Func, Call};
use crate::compiler::error::{Span, Diagnostic};
use crate::compiler::token::{Token, TokenKind};

pub struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,    
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            index: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut nodes = Vec::new();

        while self.peek_kind() != TokenKind::Eof {
            let node = self.parse_node()?;
            nodes.push(node);
        }

        Ok(Program { nodes })
    }

    pub fn parse_node(&mut self) -> Result<Node, Diagnostic> {
        match self.peek_kind() {
            // ---------- structural constructs ----------
            TokenKind::BlockStart => {
                // parse_block handles:
                // - stray block end
                // - missing closing delimiter
                // - interior parsing
                self.parse_block()
            }

            TokenKind::KwFn => {
                // parse_func handles:
                // - full function structure validation
                // - parameter rules
                // - body parsing
                self.parse_func()
            }

            // ---------- everything else ----------
            _ => self.parse_statement_entry(),
        }
    }

    fn parse_statement_entry(&mut self) -> Result<Node, Diagnostic> {
        let mut i = self.index;

        while let Some(tok) = self.tokens.get(i) {
            match tok.kind {
                // statement-defining keywords
                TokenKind::KwRet => {
                    return self.parse_ret();
                }

                // statement-defining operators
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Guard => {
                    // DO NOT consume here
                    return match tok.kind {
                        TokenKind::Define      => self.parse_define(),
                        TokenKind::DefineEmpty => self.parse_define_empty(),
                        TokenKind::Copy        => self.parse_copy(),
                        TokenKind::Bind        => self.parse_bind(),
                        TokenKind::Guard       => self.parse_guard(),
                        _ => unreachable!(),
                    };
                }

                // hard stop: statement boundary
                TokenKind::Semicolon
                | TokenKind::BlockEnd
                | TokenKind::FuncEnd => break,

                _ => i += 1,
            }
        }

        // no statement operator claimed it
        self.parse_call_statement()
    }

    fn parse_ret(&mut self) -> Result<Node, Diagnostic> {
        // We are committing to parsing a return statement
        self.bump(); // consume `ret`

        // 🔒 REQUIRED: verify semicolon exists BEFORE parsing anything else
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                return Err(
                    Diagnostic::error("unterminated return statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` to terminate this return statement.\n\
                            Examples:\n\
                            `ret;`\n\
                            `ret 42;`",
                        ),
                );
            }
        };

        // `ret;` — valid, no value
        if self.peek_kind() == TokenKind::Semicolon {
            self.bump(); // consume `;`
            return Ok(Node::Ret(Ret { value: None }));
        }

        // Disallow statement operators inside return value
        let mut i = self.index;
        while i < stmt_end {
            match self.tokens[i].kind {
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Guard
                | TokenKind::KwRet => {
                    return Err(
                        Diagnostic::error(
                            "invalid return statement",
                            Span {
                                start: self.tokens[i].pos,
                                end: self.tokens[i].pos + self.tokens[i].lexeme.len(),
                            },
                        )
                        .with_help(
                            "Return values must be a value expression or function call.\n\
                            Statements are not allowed inside `ret`.\n\
                            Example: `ret x + 1;`",
                        ),
                    );
                }
                _ => {}
            }
            i += 1;
        }

        // ✅ Structure validated — now parse the return value
        let value = self.parse_rhs()?;

        // Consume terminating semicolon
        self.bump(); // `;`

        Ok(Node::Ret(Ret {
            value: Some(Box::new(value)),
        }))
    }

    fn parse_define_empty(&mut self) -> Result<Node, Diagnostic> {

        // 1️⃣ Optional `loc` (syntactic only — no semantics here)
        let _is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump(); // consume `loc`
            true
        } else {
            false
        };

        // Identifier
        let ident_tok = self.bump().ok_or_else(|| {
            Diagnostic::error("invalid define empty statement", self.current_span())
                .with_help("Druim define empty statements must start with an identifier.")
        })?;

        if ident_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid define empty statement",
                    Span {
                        start: ident_tok.pos,
                        end: ident_tok.pos + ident_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Druim Define empty statements must begin with an identifier.\n\
                    Example: `x = 42;`",
                ),
            );
        }

        let name = ident_tok.lexeme.clone();

        // 3️⃣ Consume `=;` (operator is already known by parse_statement_entry)
        self.bump(); // consume `=;`

        // 4️⃣ Chaining is illegal: `a =; = b;` / `a =; := b;` / etc.
        match self.peek_kind() {
            TokenKind::Define
            | TokenKind::DefineEmpty
            | TokenKind::Copy
            | TokenKind::Bind
            | TokenKind::Guard => {
                return Err(
                    Diagnostic::error("invalid define empty statement", self.current_span())
                        .with_help(
                            "Druim statement operators cannot be chained.\n\
                            Split this into multiple statements.\n\
                            Example: `a =; b = 1;`",
                        ),
                );
            }
            _ => {}
        }

        Ok(Node::DefineEmpty(DefineEmpty { name }))
    }

    fn parse_define(&mut self) -> Result<Node, Diagnostic> {

        // 1️⃣ Statement MUST terminate
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                return Err(
                    Diagnostic::error("unterminated define statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` to terminate this define statement.\n\
                            Example: `x = 42;`",
                        ),
                );
            }
        };

        // 2️⃣ Optional `loc`
        let _is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump();
            true
        } else {
            false
        };

        // 3️⃣ Identifier (exactly once)
        let ident_tok = self.bump().ok_or_else(|| {
            Diagnostic::error("invalid define statement", self.current_span())
                .with_help("Druim define statements must start with an identifier.")
        })?;

        if ident_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid define statement",
                    Span {
                        start: ident_tok.pos,
                        end: ident_tok.pos + ident_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Druim define statements must begin with an identifier.\n\
                    Example: `x = 42;`",
                ),
            );
        }

        let name = ident_tok.lexeme.clone();

        // 4️⃣ Consume `=` (guaranteed by entry routing)
        self.bump();

        // 5️⃣ RHS must exist
        if self.peek_kind() == TokenKind::Semicolon {
            return Err(
                Diagnostic::error("invalid define statement", self.current_span())
                    .with_help(
                        "A define statement requires a value after `=`.\n\
                        Did you mean to use the empty define operator?\n\
                        Example: `x =;`",
                    ),
            );
        }

        // 6️⃣ Structural scan: no statement operators allowed inside RHS
        let mut i = self.index;
        while i < stmt_end {
            match self.tokens[i].kind {
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Guard => {
                    return Err(
                        Diagnostic::error(
                            "invalid define statement",
                            Span {
                                start: self.tokens[i].pos,
                                end: self.tokens[i].pos + self.tokens[i].lexeme.len(),
                            },
                        )
                        .with_help(
                            "Define statements cannot contain other statement operators.\n\
                            If you intended to assign from another identifier, use `:=`.\n\
                            Example: `a := b;`",
                        ),
                    );
                }
                _ => {}
            }
            i += 1;
        }

        // 7️⃣ RHS must not be a single identifier
        if self.index + 1 == stmt_end && self.tokens[self.index].kind == TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid define statement",
                    Span {
                        start: self.tokens[self.index].pos,
                        end: self.tokens[self.index].pos + self.tokens[self.index].lexeme.len(),
                    },
                )
                .with_help(
                    "Define statements cannot assign directly from another identifier.\n\
                    Use `:=` to copy from an identifier.\n\
                    Example: `a := b;`",
                ),
            );
        }

        // 8️⃣ Parse RHS LAST (now structurally valid)
        let value = self.parse_rhs()?;

        // 9️⃣ Consume `;`
        self.bump();

        Ok(Node::Define(Define {
            name,
            value: Box::new(value),
        }))
    }

    fn parse_copy(&mut self) -> Result<Node, Diagnostic> {

        // 1️⃣ Verify the statement is terminated with `;` BEFORE parsing structure
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                return Err(
                    Diagnostic::error("unterminated copy statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` to terminate this copy statement.\n\
                            Example: `a := b;`",
                        ),
                );
            }
        };

        // 2️⃣ Optional `loc`
        let _is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump(); // consume `loc`
            true
        } else {
            false
        };

        // 3️⃣ Left-hand identifier (single assertion)
        let lhs_tok = self.bump().unwrap_or_else(|| {
            Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                pos: self.current_span().start,
            }
        });

        if lhs_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid copy statement",
                    Span {
                        start: lhs_tok.pos,
                        end: lhs_tok.pos + lhs_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Copy statements must begin with an identifier.\n\
                    Example: `a := b;`",
                ),
            );
        }

        let name = lhs_tok.lexeme.clone();

        // 4️⃣ Consume `:=` (operator already identified by entry function)
        self.bump();

        // 5️⃣ Right-hand identifier (single assertion)
        let rhs_tok = self.bump().unwrap_or_else(|| {
            Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                pos: self.current_span().start,
            }
        });

        if rhs_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid copy statement",
                    Span {
                        start: rhs_tok.pos,
                        end: rhs_tok.pos + rhs_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Copy statements require an identifier after `:=`.\n\
                    Example: `a := b;`",
                ),
            );
        }

        let target = rhs_tok.lexeme.clone();

        // 6️⃣ Disallow chaining inside the statement boundary
        let mut i = self.index;
        while i < stmt_end {
            match self.tokens[i].kind {
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Guard => {
                    return Err(
                        Diagnostic::error(
                            "invalid copy statement",
                            Span {
                                start: self.tokens[i].pos,
                                end: self.tokens[i].pos + self.tokens[i].lexeme.len(),
                            },
                        )
                        .with_help(
                            "Copy statements cannot be chained.\n\
                            Split this into multiple statements.\n\
                            Example:\n\
                            `a := b; c := d;`",
                        ),
                    );
                }
                _ => {}
            }
            i += 1;
        }

        // 7️⃣ Consume `;`
        self.bump();

        Ok(Node::Copy(Copy { name, target }))
    }

    fn parse_bind(&mut self) -> Result<Node, Diagnostic> {

        // 1️⃣ Verify the statement is terminated with `;` BEFORE parsing structure
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                return Err(
                    Diagnostic::error("unterminated bind statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` to terminate this bind statement.\n\
                            Example: `a :> b;`",
                        ),
                );
            }
        };

        // 2️⃣ Optional `loc`
        let _is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump(); // consume `loc`
            true
        } else {
            false
        };

        // 3️⃣ Left-hand identifier (single assertion)
        let lhs_tok = self.bump().unwrap_or_else(|| {
            Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                pos: self.current_span().start,
            }
        });

        if lhs_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid bind statement",
                    Span {
                        start: lhs_tok.pos,
                        end: lhs_tok.pos + lhs_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Bind statements must begin with an identifier.\n\
                    Example: `a :> b;`",
                ),
            );
        }

        let name = lhs_tok.lexeme.clone();

        // 4️⃣ Consume `:>` (operator already identified by entry function)
        self.bump();

        // 5️⃣ Right-hand identifier (single assertion)
        let rhs_tok = self.bump().unwrap_or_else(|| {
            Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                pos: self.current_span().start,
            }
        });

        if rhs_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid bind statement",
                    Span {
                        start: rhs_tok.pos,
                        end: rhs_tok.pos + rhs_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Bind statements require an identifier after `:>`.\n\
                    Example: `a :> b;`",
                ),
            );
        }

        let target = rhs_tok.lexeme.clone();

        // 6️⃣ Disallow chaining inside the statement boundary
        let mut i = self.index;
        while i < stmt_end {
            match self.tokens[i].kind {
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Guard => {
                    return Err(
                        Diagnostic::error(
                            "invalid bind statement",
                            Span {
                                start: self.tokens[i].pos,
                                end: self.tokens[i].pos + self.tokens[i].lexeme.len(),
                            },
                        )
                        .with_help(
                            "Bind statements cannot be chained.\n\
                            Split this into multiple statements.\n\
                            Example:\n\
                            `a :> b; c :> d;`",
                        ),
                    );
                }
                _ => {}
            }
            i += 1;
        }

        // 7️⃣ Consume `;`
        self.bump();

        Ok(Node::Bind(Bind { name, target }))
    }

    fn parse_guard(&mut self) -> Result<Node, Diagnostic> {

        // 1️⃣ Find statement terminator FIRST
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                return Err(
                    Diagnostic::error("unterminated guard statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` to terminate this guard statement.\n\
                            Example: `x ?= y;`",
                        ),
                );
            }
        };

        // 2️⃣ Optional `loc` (structure only — no semantics)
        let _is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump();
            true
        } else {
            false
        };

        // 3️⃣ Identifier (REQUIRED, checked ONCE)
        let ident_tok = self.bump().ok_or_else(|| {
            Diagnostic::error("invalid guard statement", self.current_span())
                .with_help(
                    "A guard statement must begin with an identifier.\n\
                    Example: `x ?= y;`",
                )
        })?;

        if ident_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid guard statement",
                    Span {
                        start: ident_tok.pos,
                        end: ident_tok.pos + ident_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Guard statements must begin with an identifier.\n\
                    Example: `x ?= y;`",
                ),
            );
        }

        let name = ident_tok.lexeme.clone();

        // 4️⃣ Consume `?=` (we are here because entry already matched it)
        self.bump(); // consume `?=`

        // 5️⃣ First branch MUST exist
        match self.peek_kind() {
            TokenKind::Semicolon | TokenKind::Colon => {
                return Err(
                    Diagnostic::error("invalid guard statement", self.current_span())
                        .with_help(
                            "A guard statement requires a value after `?=`.\n\
                            Did you mean to use an empty define?\n\
                            Example: `x =;`",
                        ),
                );
            }
            _ => {}
        }

        // 6️⃣ Scan for illegal statement operators inside guard
        let mut i = self.index;
        while i < stmt_end {
            match self.tokens[i].kind {
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Guard => {
                    return Err(
                        Diagnostic::error(
                            "invalid guard statement",
                            Span {
                                start: self.tokens[i].pos,
                                end: self.tokens[i].pos + self.tokens[i].lexeme.len(),
                            },
                        )
                        .with_help(
                            "Guard branches must be values, not statements.\n\
                            Split this into separate statements.",
                        ),
                    );
                }
                _ => {}
            }
            i += 1;
        }

        // 7️⃣ Parse branches (value parsing LAST)
        let mut branches = Vec::new();

        // first branch
        branches.push(self.parse_rhs()?);

        // fallback branches
        while self.peek_kind() == TokenKind::Colon {
            self.bump(); // consume `:`

            if self.peek_kind() == TokenKind::Semicolon {
                return Err(
                    Diagnostic::error("invalid guard statement", self.current_span())
                        .with_help(
                            "Expected a value after `:` in guard statement.\n\
                            Example: `x ?= y : z;`",
                        ),
                );
            }

            branches.push(self.parse_rhs()?);
        }

        // 8️⃣ Consume terminator
        self.bump(); // consume `;`

        Ok(Node::Guard(Guard {
            target: name,
            branches,
        }))
    }

    fn parse_block(&mut self) -> Result<Node, Diagnostic> {

        // 2️⃣ Consume block start
        self.bump(); // `:{`

        // 3️⃣ Verify closure BEFORE parsing anything inside
        let has_end = self.tokens[self.index..]
            .iter()
            .any(|t| t.kind == TokenKind::BlockEnd);

        if !has_end {
            return Err(
                Diagnostic::error("unterminated block structure", self.current_span())
                    .with_help(
                        "Druim expected a closing block delimiter `}:`.",
                    ),
            );
        }

        // 4️⃣ Parse statements inside the validated block
        let mut nodes = Vec::new();

        while self.peek_kind() != TokenKind::BlockEnd {
            if self.peek_kind() == TokenKind::BlockChain {
                self.bump(); // `}{`
                continue;
            }

            nodes.push(self.parse_statement_entry()?);
        }

        // 5️⃣ Consume closing delimiter
        self.bump(); // `}:`

        Ok(Node::Block(Block { nodes }))
    }

    fn parse_func(&mut self) -> Result<Node, Diagnostic> {
        // ─────────────────────────────────────────────
        //  Consume `fn`
        // ─────────────────────────────────────────────
        self.bump(); // `fn`

        // ─────────────────────────────────────────────
        //  Verify function CAN close (structure-first)
        // ─────────────────────────────────────────────
        let has_end = self.tokens[self.index..]
            .iter()
            .any(|t| t.kind == TokenKind::FuncEnd);

        if !has_end {
            return Err(
                Diagnostic::error("unterminated function structure", self.current_span())
                    .with_help(
                        "Druim expected a closing function delimiter `):`.",
                    ),
            );
        }

        // ─────────────────────────────────────────────
        //  Function name (REQUIRED)
        // ─────────────────────────────────────────────
        let name_tok = self.bump().ok_or_else(|| {
            Diagnostic::error("invalid function structure", self.current_span())
                .with_help("Druim expected a function name after the `fn` keyword.")
        })?;

        if name_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid function structure",
                    Span {
                        start: name_tok.pos,
                        end: name_tok.pos + name_tok.lexeme.len(),
                    },
                )
                .with_help("Druim expected a function name after the `fn` keyword."),
            );
        }

        let name = name_tok.lexeme.clone();

        if !is_snake_case(&name) {
            return Err(
                Diagnostic::error(
                    "invalid function name",
                    Span {
                        start: name_tok.pos,
                        end: name_tok.pos + name_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Function names in Druim must use snake_case (lowercase letters and underscores).",
                ),
            );
        }

        // ─────────────────────────────────────────────
        //  Parameter block MUST exist
        // ─────────────────────────────────────────────
        if self.peek_kind() != TokenKind::FuncStart {
            return Err(
                Diagnostic::error("invalid function structure", self.current_span())
                    .with_help(
                        "Druim expected a parameter block starting with `:(` after the function name.",
                    ),
            );
        }

        self.bump(); // consume `:(`

        // ─────────────────────────────────────────────
        //  Verify AT LEAST ONE BODY EXISTS (structure only)
        // ─────────────────────────────────────────────
        {
            let mut i = self.index;

            // Skip parameter tokens until first `)(`
            while i < self.tokens.len() && self.tokens[i].kind != TokenKind::FuncChain {
                i += 1;
            }

            if i >= self.tokens.len() {
                unreachable!("FuncEnd existence was already verified");
            }

            // Move past first `)(`
            i += 1;

            if self.tokens[i].kind == TokenKind::FuncEnd {
                return Err(
                    Diagnostic::error("incomplete function definition", self.current_span())
                        .with_help(
                            "Druim requires at least one function body before the closing `):`.",
                        ),
                );
            }
        }

        // ─────────────────────────────────────────────
        //  Parse PARAMETERS (now allowed)
        // ─────────────────────────────────────────────
        let mut params = Vec::new();

        if self.peek_kind() != TokenKind::FuncChain {
            loop {
                if self.peek_kind() == TokenKind::KwLoc {
                    return Err(
                        Diagnostic::error("invalid function parameter", self.current_span())
                            .with_help("`loc` is not allowed in function parameter declarations."),
                    );
                }

                let ident_tok = self.bump().ok_or_else(|| {
                    Diagnostic::error("invalid function parameter", self.current_span())
                        .with_help("Druim expected a parameter name.")
                })?;

                if ident_tok.kind != TokenKind::Ident {
                    return Err(
                        Diagnostic::error(
                            "invalid function parameter",
                            Span {
                                start: ident_tok.pos,
                                end: ident_tok.pos + ident_tok.lexeme.len(),
                            },
                        )
                        .with_help(
                            "Function parameters must begin with an identifier.\n\
                            Examples: `x`, `x = 10`",
                        ),
                    );
                }

                let param_name = ident_tok.lexeme.clone();

                // Default value
                if self.peek_kind() == TokenKind::Define {
                    self.bump(); // `=`

                    if self.peek_kind() == TokenKind::Comma
                        || self.peek_kind() == TokenKind::FuncChain
                    {
                        return Err(
                            Diagnostic::error("invalid default parameter", self.current_span())
                                .with_help(
                                    "Default parameters require a value.\n\
                                    Example: `x = 10`",
                                ),
                        );
                    }

                    let value = self.parse_rhs()?;

                    params.push(Param {
                        name: param_name,
                        default: Some(value),
                    });
                } else {
                    params.push(Param {
                        name: param_name,
                        default: None,
                    });
                }

                match self.peek_kind() {
                    TokenKind::Comma => {
                        self.bump();
                        continue;
                    }
                    TokenKind::FuncChain => break,
                    _ => {
                        return Err(
                            Diagnostic::error("invalid function parameter list", self.current_span())
                                .with_help(
                                    "Parameters must be separated by commas and terminated with `)(`.",
                                ),
                        );
                    }
                }
            }
        }

        self.bump(); // consume `)(`

        // ─────────────────────────────────────────────
        //  Parse FUNCTION BODIES (statements allowed now)
        // ─────────────────────────────────────────────
        let mut bodies = Vec::new();

        loop {
            let mut nodes = Vec::new();

            while self.peek_kind() != TokenKind::FuncChain
                && self.peek_kind() != TokenKind::FuncEnd
            {
                nodes.push(self.parse_statement_entry()?);
            }

            bodies.push(Node::Block(Block { nodes }));

            if self.peek_kind() == TokenKind::FuncChain {
                self.bump(); // `)(`
                continue;
            }

            break;
        }

        self.bump(); // consume `):`

        Ok(Node::Func(Func {
            name,
            params,
            bodies,
        }))
    }

    fn parse_rhs(&mut self) -> Result<Node, Diagnostic> {
        let start_span = self.current_span();

        // Explicit call detection
        if self.peek_kind() == TokenKind::Ident {
            if let Some(next) = self.tokens.get(self.index + 1) {
                if next.kind == TokenKind::LParen {
                    return self.parse_call();
                }
            }
        }

        let value = self.parse_expr()?;

        // Bare identifiers are not values
        if matches!(value, Node::Ident(_)) {
            return Err(
                Diagnostic::error("invalid value expression", start_span)
                    .with_help(
                        "A bare identifier is not a value.\n\
                        Use a function call, copy (`:=`), or bind (`:>`) instead.",
                    ),
            );
        }

        Ok(value)
    }

    fn parse_call(&mut self) -> Result<Node, Diagnostic> {
        // ─────────────────────────────────────────────
        // 1️⃣ Callee (identifier only, for now)
        // ─────────────────────────────────────────────
        let callee_tok = self.bump().ok_or_else(|| {
            Diagnostic::error("unexpected end of input", self.current_span())
                .with_help("Druim expected a function call.")
        })?;

        let callee = match callee_tok.kind {
            TokenKind::Ident => callee_tok.lexeme.clone(),
            _ => {
                return Err(
                    Diagnostic::error(
                        "invalid function call",
                        Span {
                            start: callee_tok.pos,
                            end: callee_tok.pos + callee_tok.lexeme.len(),
                        },
                    )
                    .with_help(
                        "Druim expected a function name before the call parentheses.\n\
                        Example: `foo(1, 2)`",
                    ),
                );
            }
        };

        // ─────────────────────────────────────────────
        // 2️⃣ Require opening parenthesis
        // ─────────────────────────────────────────────
        if self.peek_kind() != TokenKind::LParen {
            return Err(
                Diagnostic::error("invalid function call", self.current_span())
                    .with_help(
                        "Druim expected `(` after the function name.\n\
                        Example: `foo(1)`",
                    ),
            );
        }

        self.bump(); // consume '('

        // ─────────────────────────────────────────────
        // 3️⃣ Arguments (value-only)
        // ─────────────────────────────────────────────
        let mut args = Vec::new();

        if self.peek_kind() != TokenKind::RParen {
            loop {
                args.push(self.parse_rhs()?);

                match self.peek_kind() {
                    TokenKind::Comma => {
                        self.bump();
                    }
                    TokenKind::RParen => break,
                    _ => {
                        let span = self.current_span();
                        return Err(
                            Diagnostic::error("invalid function call", span)
                                .with_help(
                                    "Function arguments must be separated by commas and closed with `)`.",
                                ),
                        );
                    }
                }
            }
        }

        // ─────────────────────────────────────────────
        // 4️⃣ Closing parenthesis
        // ─────────────────────────────────────────────
        self.bump(); // consume ')'

        Ok(Node::Call(Call {
            callee: Box::new(Node::Ident(callee)),
            args,
        }))
    }

    fn parse_call_statement(&mut self) -> Result<Node, Diagnostic> {
        // 1️⃣ REQUIRED: verify statement terminates
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                return Err(
                    Diagnostic::error(
                        "unterminated function call statement",
                        self.current_span(),
                    )
                    .with_help(
                        "Druim expected a semicolon `;` to terminate this function call.\n\
                        Example: `do_work();`",
                    ),
                );
            }
        };

        // 2️⃣ Must start with identifier
        let ident_tok = self.bump().ok_or_else(|| {
            Diagnostic::error(
                "invalid function call statement",
                self.current_span(),
            )
            .with_help(
                "A function call statement must begin with a function name.\n\
                Example: `do_work();`",
            )
        })?;

        if ident_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid function call statement",
                    Span {
                        start: ident_tok.pos,
                        end: ident_tok.pos + ident_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Function call statements must begin with an identifier.\n\
                    Example: `do_work();`",
                ),
            );
        }

        // 3️⃣ Must be immediately followed by `(`
        if self.peek_kind() != TokenKind::LParen {
            return Err(
                Diagnostic::error(
                    "invalid function call statement",
                    self.current_span(),
                )
                .with_help(
                    "A bare identifier is not a valid statement.\n\
                    Did you mean to call a function?\n\
                    Example: `do_work();`",
                ),
            );
        }

        // 4️⃣ Scan for illegal chaining BEFORE parsing call
        let mut i = self.index;
        while i < stmt_end {
            match self.tokens[i].kind {
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Guard => {
                    return Err(
                        Diagnostic::error(
                            "invalid function call statement",
                            Span {
                                start: self.tokens[i].pos,
                                end: self.tokens[i].pos + self.tokens[i].lexeme.len(),
                            },
                        )
                        .with_help(
                            "Function call statements cannot be chained with other statement operators.\n\
                            Split this into multiple statements.",
                        ),
                    );
                }
                _ => {}
            }
            i += 1;
        }

        // 5️⃣ Now it is safe to parse the call
        let call = self.parse_call()?; // produces Node::Call

        // 6️⃣ Consume semicolon
        self.bump();

        Ok(call)
    }

    pub fn parse_expr(&mut self) -> Result<Node, Diagnostic> {
        self.parse_bp(0)
    }

    // ===== Pratt parser =====

    fn parse_bp(&mut self, min_bp: u8) -> Result<Node, Diagnostic> {
        let mut lhs = self.parse_prefix()?; // now returns Node

        loop {
            let op = self.peek_kind();

            let Some((l_bp, r_bp, infix_kind)) = infix_binding_power(op) else {
                break;
            };

            if l_bp < min_bp {
                break;
            }

            // consume operator
            self.bump();

            let rhs = self.parse_bp(r_bp)?;
            lhs = build_infix(infix_kind, lhs, rhs); // returns Node
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Node, Diagnostic> {
        let span_start = self.current_span().start;

        let tok = self.bump().ok_or_else(|| {
            Diagnostic::error(
                "unexpected end of input",
                Span { start: span_start, end: span_start },
            )
            .with_help("Druim expected a value expression here.")
        })?;

        match tok.kind {
            // ─── Atoms ──────────────────────────────
            TokenKind::Ident => Ok(Node::Ident(tok.lexeme.clone())),

            TokenKind::NumLit => {
                let n = tok.lexeme.parse::<i64>().unwrap_or(0);
                Ok(Node::Lit(Literal::Num(n)))
            }

            TokenKind::DecLit => Ok(Node::Lit(Literal::Dec(tok.lexeme.clone()))),

            TokenKind::TextLit => Ok(Node::Lit(Literal::Text(tok.lexeme.clone()))),

            TokenKind::KwVoid => Ok(Node::Lit(Literal::Void)),

            // ─── Unary operators ────────────────────
            TokenKind::Not => {
                let rhs = self.parse_bp(PREFIX_BP)?;
                Ok(Node::Not(Box::new(rhs)))
            }

            TokenKind::Sub => {
                let rhs = self.parse_bp(PREFIX_BP)?;
                Ok(Node::Neg(Box::new(rhs)))
            }

            // ─── Grouping ───────────────────────────
            TokenKind::LParen => {
                let expr = self.parse_bp(0)?;
                self.expect(TokenKind::RParen, "`)`")?;
                Ok(expr)
            }

            // ─── Explicitly illegal value starters ──
            TokenKind::Define
            | TokenKind::DefineEmpty
            | TokenKind::Copy
            | TokenKind::Bind
            | TokenKind::Guard => {
                Err(
                    Diagnostic::error(
                        "invalid value expression",
                        Span {
                            start: tok.pos,
                            end: tok.pos + tok.lexeme.len(),
                        },
                    )
                    .with_help(
                        "Statement operators are not valid values.\n\
                        Use them as complete statements ending with `;`.",
                    ),
                )
            }

            TokenKind::KwFn
            | TokenKind::KwLoc
            | TokenKind::KwRet
            | TokenKind::BlockStart => {
                Err(
                    Diagnostic::error(
                        "invalid value expression",
                        Span {
                            start: tok.pos,
                            end: tok.pos + tok.lexeme.len(),
                        },
                    )
                    .with_help(
                        "This construct cannot be used as a value.\n\
                        It must appear in its own statement context.",
                    ),
                )
            }

            // ─── Everything else ────────────────────
            _ => Err(
                Diagnostic::error(
                    "unexpected token in value expression",
                    Span {
                        start: tok.pos,
                        end: tok.pos + tok.lexeme.len(),
                    },
                )
                .with_help("Druim expected a value here."),
            ),
        }
    }

    fn expect(&mut self, kind: TokenKind, expected: &'static str) -> Result<(), Diagnostic> {
        let span_start = self.current_span().start;
        let tok = self.bump().ok_or_else(|| {
            Diagnostic::error(
                "unexpected end of input",
                Span { start: span_start, end: span_start },
            )
            .with_help(expected)
        })?;

        if tok.kind != kind {
            return Err(
                Diagnostic::error(
                    "unexpected token",
                    Span {
                        start: tok.pos,
                        end: tok.pos + tok.lexeme.len(),
                    },
                )
                .with_help(expected)
            );
        }

        Ok(())
    }

    fn bump(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.index);
        if t.is_some() {
            self.index += 1;
        }
        t
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn peek_kind(&self) -> TokenKind {
        self.peek().map(|t| t.kind).unwrap_or(TokenKind::Eof)
    }

    fn current_span(&self) -> Span {
        if let Some(tok) = self.peek() {
            Span {
                start: tok.pos,
                end: tok.pos + tok.lexeme.len(),
            }
        } else if let Some(prev) = self.tokens.last() {
            let end = prev.pos + prev.lexeme.len();
            Span { start: end, end }
        } else {
            Span { start: 0, end: 0 }
        }
    }
}

fn is_snake_case(name: &str) -> bool {
    let mut prev_underscore = false;

    for c in name.chars() {
        if c == '_' {
            if prev_underscore {
                return false;
            }
            prev_underscore = true;
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
            prev_underscore = false;
        } else {
            return false;
        }
    }

    !name.starts_with('_') && !name.ends_with('_')
}

const PREFIX_BP: u8 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Infix {
    // Call
    Call,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,

    // Colon semantics
    Has,
    Present,

    // Flow
    Pipe,
}

fn infix_binding_power(op: TokenKind) -> Option<(u8, u8, Infix)> {
    use Infix::*;

    Some(match op {
        // call binds tight: f(x)
        TokenKind::LParen => (95, 96, Call),

        // arithmetic
        TokenKind::Mul => (70, 71, Mul),
        TokenKind::Div => (70, 71, Div),
        TokenKind::Mod => (70, 71, Mod),

        TokenKind::Add => (60, 61, Add),
        TokenKind::Sub => (60, 61, Sub),

        // comparison
        TokenKind::Lt => (50, 51, Lt),
        TokenKind::Le => (50, 51, Le),
        TokenKind::Gt => (50, 51, Gt),
        TokenKind::Ge => (50, 51, Ge),

        TokenKind::Eq => (45, 46, Eq),
        TokenKind::Ne => (45, 46, Ne),

        // logical
        TokenKind::And => (30, 31, And),
        TokenKind::Or => (25, 26, Or),

        // colon family
        TokenKind::Has => (22, 23, Has),
        TokenKind::Present => (22, 23, Present),

        // pipe
        TokenKind::Pipe => (20, 21, Pipe),

        _ => return None,
    })
}

fn build_infix(kind: Infix, lhs: Node, rhs: Node) -> Node {
    use Infix::*;

    match kind {
        Add => Node::Add(Box::new(lhs), Box::new(rhs)),
        Sub => Node::Sub(Box::new(lhs), Box::new(rhs)),
        Mul => Node::Mul(Box::new(lhs), Box::new(rhs)),
        Div => Node::Div(Box::new(lhs), Box::new(rhs)),
        Mod => Node::Mod(Box::new(lhs), Box::new(rhs)),

        Eq => Node::Eq(Box::new(lhs), Box::new(rhs)),
        Ne => Node::Ne(Box::new(lhs), Box::new(rhs)),
        Lt => Node::Lt(Box::new(lhs), Box::new(rhs)),
        Le => Node::Le(Box::new(lhs), Box::new(rhs)),
        Gt => Node::Gt(Box::new(lhs), Box::new(rhs)),
        Ge => Node::Ge(Box::new(lhs), Box::new(rhs)),

        And => Node::And(Box::new(lhs), Box::new(rhs)),
        Or => Node::Or(Box::new(lhs), Box::new(rhs)),

        Has => Node::Has(Box::new(lhs), Box::new(rhs)),
        Present => Node::Present(Box::new(lhs), Box::new(rhs)),

        Pipe => Node::Pipe(Box::new(lhs), Box::new(rhs)),

        Call => unreachable!("Call is handled in parse_bp"),
    }
}
