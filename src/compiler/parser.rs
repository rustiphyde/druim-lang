use crate::compiler::ast::{
    Bind, Block, BlockSegment, Call, Copy, Define, DefineEmpty, Func,
    Guard, GuardBranch, Literal, Node, Param, Program, Ret,
};
use crate::compiler::error::{Span, Diagnostic};
use crate::compiler::token::{Token, TokenKind};

pub struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
    in_block: bool,
    in_func: bool, 
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            index: 0,
            in_block: false,
            in_func: false,
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
        let value =
            if self.index + 1 == stmt_end
                && self.peek_kind() == TokenKind::Ident
            {
                let ident = self.bump().expect("identifier token must exist");
                Node::Ident(ident.lexeme.clone())
            } else {
                self.parse_rhs()?
            };

        // Consume terminating semicolon
        self.bump(); // `;`

        Ok(Node::Ret(Ret {
            value: Some(Box::new(value)),
        }))
    }

    fn parse_define_empty(&mut self) -> Result<Node, Diagnostic> {

        // Optional `loc` (syntactic only — no semantics here)
        let is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump();
            true
        } else {
            false
        };

        // Identifier (single assertion)
        let ident_tok = match self.bump() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("invalid empty definition", self.current_span())
                        .with_help(
                            "Druim empty definitions must begin with an identifier.\n\
                            Example: `x =;`",
                        ),
                );
            }
        };

        if ident_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid empty definition",
                    Span {
                        start: ident_tok.pos,
                        end: ident_tok.pos + ident_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Druim empty definitions must begin with an identifier.\n\
                    Example: `x =;`",
                ),
            );
        }

        let name = ident_tok.lexeme.clone();

        // Consume `=;` (operator already identified by entry function)
        self.bump();

        // Chaining is illegal
        match self.peek_kind() {
            TokenKind::Define
            | TokenKind::DefineEmpty
            | TokenKind::Copy
            | TokenKind::Bind
            | TokenKind::Guard => {
                return Err(
                    Diagnostic::error("invalid empty definition", self.current_span())
                        .with_help(
                            "Statement operators cannot be chained.\n\
                            Split this into multiple statements.\n\
                            Example: `a =; b = 1;`",
                        ),
                );
            }
            _ => {}
        }

        let node = Node::DefineEmpty(DefineEmpty { name });

        if is_local {
            Ok(Node::Local(Box::new(node)))
        } else {
            Ok(node)
        }
    }

    fn parse_define(&mut self) -> Result<Node, Diagnostic> {
        // Statement MUST terminate
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

        // Optional `loc`
        let is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump();
            true
        } else {
            false
        };

        // Identifier (single assertion)
        let ident_tok = match self.bump() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("invalid define statement", self.current_span())
                        .with_help(
                            "Druim define statements must begin with an identifier.\n\
                            Example: `x = 42;`",
                        ),
                );
            }
        };

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

        // Consume `=` (guaranteed by entry routing)
        self.bump();

        // RHS must exist
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

        // Structural scan: no statement operators allowed inside RHS
        let mut i = self.index;
        while i < stmt_end {
            match self.tokens[i].kind {
                TokenKind::Define => {
                    return Err(
                        Diagnostic::error(
                            "invalid define statement",
                            Span {
                                start: self.tokens[i].pos,
                                end: self.tokens[i].pos + self.tokens[i].lexeme.len(),
                            },
                        )
                        .with_help(
                            "Define statements cannot be chained.\n\
                            Split this into multiple statements.\n\
                            Example: `a = 1; b = 2;`",
                        ),
                    );
                }

                TokenKind::DefineEmpty
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
                            Split this into separate statements.",
                        ),
                    );
                }

                _ => {}
            }

            i += 1;
        }

        // RHS must not be a single identifier
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
                    "Define statements cannot define directly from another identifier.\n\
                    Use `:=` to copy a value or `:>` to create a live binding.\n\
                    Examples: `a := b;` or `a :> b;`",
                ),
            );
        }

        // Parse RHS LAST
        let value = self.parse_rhs()?;

        // The parsed expression must consume the entire RHS.
        // Only the terminating semicolon may remain.
        let next_tok = match self.peek() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("unterminated define statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` after the defined value.\n\
                            Example: `x = 42;`",
                        ),
                );
            }
        };

        if next_tok.kind != TokenKind::Semicolon {
            return Err(
                Diagnostic::error(
                    "invalid define statement",
                    Span {
                        start: next_tok.pos,
                        end: next_tok.pos + next_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "A Druim define statement must contain exactly one complete expression.\n\
                    Unexpected tokens remain after the defined value.\n\
                    Example: `x = 12 + 13;`",
                ),
            );
        }

        // Consume `;`
        self.bump();

        let node = Node::Define(Define {
            name,
            value: Box::new(value),
        });

        if is_local {
            Ok(Node::Local(Box::new(node)))
        } else {
            Ok(node)
        }
    }

    fn parse_copy(&mut self) -> Result<Node, Diagnostic> {

        match self.tokens[self.index..]
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
        let is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump();
            true
        } else {
            false
        };

        // 3️⃣ Left-hand identifier (single assertion)
        let lhs_tok = match self.bump() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("invalid copy statement", self.current_span())
                        .with_help(
                            "Copy statements must begin with an identifier.\n\
                            Example: `a := b;`",
                        ),
                );
            }
        };

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
        let rhs_tok = match self.bump() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("invalid copy statement", self.current_span())
                        .with_help(
                            "Copy statements require an identifier after `:=`.\n\
                            Example: `a := b;`",
                        ),
                );
            }
        };

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

        let next_tok = match self.peek() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("unterminated copy statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` after the copy target.\n\
                            Example: `a := b;`",
                        ),
                );
            }
        };

        if next_tok.kind != TokenKind::Semicolon {
            let is_chained = matches!(
                next_tok.kind,
                TokenKind::Define
                    | TokenKind::DefineEmpty
                    | TokenKind::Copy
                    | TokenKind::Bind
                    | TokenKind::Guard
            );

            let diagnostic = Diagnostic::error(
                "invalid copy statement",
                Span {
                    start: next_tok.pos,
                    end: next_tok.pos + next_tok.lexeme.len(),
                },
            );

            return if is_chained {
                Err(diagnostic.with_help(
                    "Copy statements cannot be chained.\n\
                    Split this into multiple statements.\n\
                    Example:\n\
                    `a := b; c := d;`",
                ))
            } else {
                Err(diagnostic.with_help(
                    "Copy statements must end immediately after the target identifier.\n\
                    Druim expected `;` after `b`.\n\
                    Example: `a := b;`",
                ))
            };
        }
        // 7️⃣ Consume `;`
        self.bump();

        let node = Node::Copy(Copy { name, target });

        if is_local {
            Ok(Node::Local(Box::new(node)))
        } else {
            Ok(node)
        }
    }

    fn parse_bind(&mut self) -> Result<Node, Diagnostic> {
        match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(_) => {}
            None => {
                return Err(
                    Diagnostic::error("unterminated bind statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` to terminate this bind statement.\n\
                            Example: `a :> b;`",
                        ),
                );
            }
        }

        // Optional `loc`
        let is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump();
            true
        } else {
            false
        };

        // Left-hand identifier
        let lhs_tok = match self.bump() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("invalid bind statement", self.current_span())
                        .with_help(
                            "Bind statements must begin with an identifier.\n\
                            Example: `a :> b;`",
                        ),
                );
            }
        };

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

        // consume `:>`
        self.bump();

        // Right-hand identifier
        let rhs_tok = match self.bump() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("invalid bind statement", self.current_span())
                        .with_help(
                            "Bind statements require an identifier after `:>`.\n\
                            Example: `a :> b;`",
                        ),
                );
            }
        };

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

        // After the RHS identifier, only `;` is valid
        let next_tok = match self.peek() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("unterminated bind statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` after the bind target.\n\
                            Example: `a :> b;`",
                        ),
                );
            }
        };

        if next_tok.kind != TokenKind::Semicolon {
            let is_chained = matches!(
                next_tok.kind,
                TokenKind::Define
                    | TokenKind::DefineEmpty
                    | TokenKind::Copy
                    | TokenKind::Bind
                    | TokenKind::Guard
            );

            let diagnostic = Diagnostic::error(
                "invalid bind statement",
                Span {
                    start: next_tok.pos,
                    end: next_tok.pos + next_tok.lexeme.len(),
                },
            );

            return if is_chained {
                Err(diagnostic.with_help(
                    "Bind statements cannot be chained.\n\
                    Split this into multiple statements.\n\
                    Example:\n\
                    `a :> b; c :> d;`",
                ))
            } else {
                Err(diagnostic.with_help(
                    "Bind statements must end immediately after the target identifier.\n\
                    Example: `a :> b;`",
                ))
            };
        }

        // Consume `;`
        self.bump();

        let node = Node::Bind(Bind { name, target });

        if is_local {
            Ok(Node::Local(Box::new(node)))
        } else {
            Ok(node)
        }
    }

    fn parse_guard(&mut self) -> Result<Node, Diagnostic> {
        // Find statement terminator FIRST
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

        // Optional `loc`
        let is_local = if self.peek_kind() == TokenKind::KwLoc {
            self.bump();
            true
        } else {
            false
        };

        // Identifier (single assertion)
        let ident_tok = match self.bump() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("invalid guard statement", self.current_span())
                        .with_help(
                            "Druim guard statements must begin with an identifier.\n\
                            Example: `x ?= y;`",
                        ),
                );
            }
        };

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
                    "Druim guard statements must begin with an identifier.\n\
                    Example: `x ?= y;`",
                ),
            );
        }

        let name = ident_tok.lexeme.clone();

        // Consume `?=` (entry routing guarantees it)
        self.bump();

        // First branch must exist
        match self.peek_kind() {
            TokenKind::Semicolon | TokenKind::Colon => {
                return Err(
                    Diagnostic::error("invalid guard statement", self.current_span())
                        .with_help(
                            "A Druim guard statement requires a value after `?=`.\n\
                            Did you mean to use the DefineEmpty operator?\n\
                            Example: `x =;`",
                        )
                );
            }
            _ => {}
        }

        // Structural scan: no statement operators inside guard
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
                            "Druim guard branches must be values, not statements.\n\
                            Split this into separate statements.",
                        ),
                    );
                }
                _ => {}
            }
            i += 1;
        }

        // Parse branches LAST
        let mut branches = Vec::new();

        branches.push(GuardBranch {
            expr: self.parse_expr()?,
        });

        while self.peek_kind() == TokenKind::Colon {
            self.bump(); // consume `:`

            if self.peek_kind() == TokenKind::Semicolon {
                return Err(
                    Diagnostic::error("invalid guard statement", self.current_span())
                        .with_help(
                            "Druim expected a value after `:` in guard statement.\n\
                            Example: `x ?= y : z;`",
                        ),
                );
            }

            branches.push(GuardBranch {
                expr: self.parse_expr()?,
            });
        }

        // The final branch must consume the complete guard RHS.
        // Only the terminating semicolon may remain.
        let next_tok = match self.peek() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error("unterminated guard statement", self.current_span())
                        .with_help(
                            "Druim expected a semicolon `;` after the final guard branch.\n\
                            Example: `x ?= y : z;`",
                        ),
                );
            }
        };

        if next_tok.kind != TokenKind::Semicolon {
            return Err(
                Diagnostic::error(
                    "invalid guard statement",
                    Span {
                        start: next_tok.pos,
                        end: next_tok.pos + next_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Each Druim guard branch must contain exactly one complete expression.\n\
                    Unexpected tokens remain after the final branch.\n\
                    Example: `x ?= 12 : 13;`",
                ),
            );
        }

        // Consume `;`
        self.bump();

        let node = Node::Guard(Guard {
            target: name,
            branches,
        });

        if is_local {
            Ok(Node::Local(Box::new(node)))
        } else {
            Ok(node)
        }
    }

    fn parse_block(&mut self) -> Result<Node, Diagnostic> {
        if self.in_func {
            return Err(
                Diagnostic::error("block not allowed in function body", self.current_span())
                    .with_help(
                        "Blocks cannot appear inside function bodies.\n\
                        Use chained blocks at the top level instead.",
                    ),
            );
        }

        if self.in_block {
            return Err(
                Diagnostic::error("nested block not allowed", self.current_span())
                    .with_help(
                        "Druim blocks may be chained but not nested.\n\
                        Use `}{` to create a new block at the same level.",
                    ),
            );
        }

        // Consume block start
        self.bump(); // `:{`

        // Enter block context
        let prev = self.in_block;
        self.in_block = true;

        // Verify block can close before parsing contents
        let has_end = self.tokens[self.index..]
            .iter()
            .any(|t| t.kind == TokenKind::BlockEnd);

        if !has_end {
            self.in_block = prev;

            return Err(
                Diagnostic::error("unterminated block structure", self.current_span())
                    .with_help("Druim expected a closing block delimiter `}:`."),
            );
        }

        // Parse block-chain segments
        let mut segments = Vec::new();
        let mut nodes = Vec::new();

        while self.peek_kind() != TokenKind::BlockEnd {
            if self.peek_kind() == TokenKind::BlockChain {
                self.bump(); // `}{`

                segments.push(BlockSegment { nodes });
                nodes = Vec::new();

                continue;
            }

            nodes.push(self.parse_statement_entry()?);
        }

        // Store the final segment
        segments.push(BlockSegment { nodes });

        // Consume closing delimiter
        self.bump(); // `}:`

        // Exit block context
        self.in_block = prev;

        Ok(Node::Block(Block { segments }))
    }

    fn parse_func(&mut self) -> Result<Node, Diagnostic> {
        if self.in_func {
            return Err(
                Diagnostic::error("nested function not allowed", self.current_span())
                    .with_help(
                        "Functions cannot be defined inside other functions.\n\
                        Define functions at the top level and call them instead.",
                    ),
            );
        }

        let prev_in_func = self.in_func;
        self.in_func = true;

        let result = (|| {
            // Consume `fn`
            self.bump();

            // Verify function can close
            if !self.tokens[self.index..]
                .iter()
                .any(|t| t.kind == TokenKind::FuncEnd)
            {
                return Err(
                    Diagnostic::error("unterminated function structure", self.current_span())
                        .with_help("Druim expected a closing function delimiter `):`."),
                );
            }

            // Function name
            let name_tok = match self.bump() {
                Some(tok) => tok,
                None => {
                    return Err(
                        Diagnostic::error("invalid function structure", self.current_span())
                            .with_help("Druim expected a function name after the `fn` keyword."),
                    );
                }
            };

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

            // Parameter block must start
            if self.peek_kind() != TokenKind::FuncStart {
                return Err(
                    Diagnostic::error("invalid function structure", self.current_span())
                        .with_help(
                            "Druim expected a parameter block starting with `:(` after the function name.",
                        ),
                );
            }

            self.bump(); // consume `:(`

            // Verify at least one body delimiter exists
            let mut i = self.index;
            let mut saw_body = false;

            while i < self.tokens.len() {
                match self.tokens[i].kind {
                    TokenKind::FuncChain => {
                        saw_body = true;
                        break;
                    }
                    TokenKind::FuncEnd => break,
                    _ => {}
                }
                i += 1;
            }

            if !saw_body {
                return Err(
                    Diagnostic::error("incomplete function definition", self.current_span())
                        .with_help(
                            "Druim functions must consist of a parameter list and at least one body.\n\
                            An empty list and empty body is allowed, but a body delimiter `)(` is required.\n\
                            Example: `fn f :()():`",
                        ),
                );
            }

            // Parse parameters
            let mut params = Vec::new();

            if self.peek_kind() != TokenKind::FuncChain {
                loop {
                    if self.peek_kind() == TokenKind::KwLoc {
                        return Err(
                            Diagnostic::error("invalid function parameter", self.current_span())
                                .with_help("`loc` is not allowed in Druim function parameter declarations."),
                        );
                    }

                    let ident_tok = match self.bump() {
                        Some(tok) => tok,
                        None => {
                            return Err(
                                Diagnostic::error("invalid function parameter", self.current_span())
                                    .with_help("Druim expected a parameter name."),
                            );
                        }
                    };

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
                                "Druim function parameters must begin with an identifier.\n\
                                Examples: `x`, `x = 10`",
                            ),
                        );
                    }

                    let param_name = ident_tok.lexeme.clone();

                    if self.peek_kind() == TokenKind::Define {
                        self.bump();

                        if self.peek_kind() == TokenKind::Comma
                            || self.peek_kind() == TokenKind::FuncChain
                        {
                            return Err(
                                Diagnostic::error("invalid default parameter", self.current_span())
                                    .with_help(
                                        "Druim default parameters require a value.\n\
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
                        }
                        TokenKind::FuncChain => break,
                        _ => {
                            return Err(
                                Diagnostic::error("invalid function parameter list", self.current_span())
                                    .with_help(
                                        "Druim parameters must be separated by commas and terminated with `)(`.",
                                    ),
                            );
                        }
                    }
                }
            }

            self.bump(); // consume `)(`

            // Reject function chaining
            if self.peek_kind() == TokenKind::FuncChain {
                return Err(
                    Diagnostic::error("function chaining not allowed", self.current_span())
                        .with_help(
                            "Functions may only define a single body.\n\
                            Function chaining is not supported.",
                        ),
                );
            }

            // Parse exactly one body
            let mut nodes = Vec::new();

            while self.peek_kind() != TokenKind::FuncEnd {
                nodes.push(self.parse_statement_entry()?);
            }

            self.bump(); // consume `):`

            Ok(Node::Func(Func {
                name,
                params,
                body: nodes,
            }))
        })();

        self.in_func = prev_in_func;
        result
    }

    fn parse_rhs(&mut self) -> Result<Node, Diagnostic> {
        let start_span = self.current_span();

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

    fn parse_call_statement(&mut self) -> Result<Node, Diagnostic> {
        // Verify statement terminates
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

        // Scan for illegal statement operators before parsing
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
                            "Druim function call statements cannot be chained with other statement operators.\n\
                            Split this into multiple statements.",
                        ),
                    );
                }

                _ => {}
            }

            i += 1;
        }

        // Parse the complete call expression
        let call = self.parse_expr()?;

        // A standalone expression must structurally be a function call
        if !matches!(call, Node::Call(_)) {
            return Err(
                Diagnostic::error(
                    "invalid function call statement",
                    self.current_span(),
                )
                .with_help(
                    "Only function calls may appear as standalone expressions.\n\
                    Example: `do_work();`",
                ),
            );
        }

        // Ensure the entire statement was consumed
        if self.index != stmt_end {
            return Err(
                Diagnostic::error(
                    "invalid function call statement",
                    self.current_span(),
                )
                .with_help(
                    "A standalone function call cannot be followed by another expression.\n\
                    Split this into separate statements.",
                ),
            );
        }

        self.bump(); // consume `;`

        Ok(call)
    }

    pub fn parse_expr(&mut self) -> Result<Node, Diagnostic> {
        self.parse_bp(0)
    }

    // ===== Pratt parser =====

    fn parse_bp(&mut self, min_bp: u8) -> Result<Node, Diagnostic> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Postfix function call: lhs(...)
            if self.peek_kind() == TokenKind::LParen {
                let call_bp = 95;

                if call_bp < min_bp {
                    break;
                }

                self.bump(); // consume `(`

                let mut args = Vec::new();

                if self.peek_kind() != TokenKind::RParen {
                    loop {
                        args.push(self.parse_expr()?);

                        match self.peek_kind() {
                            TokenKind::Comma => {
                                self.bump();
                            }

                            TokenKind::RParen => break,

                            _ => {
                                return Err(
                                    Diagnostic::error(
                                        "invalid function call",
                                        self.current_span(),
                                    )
                                    .with_help(
                                        "Druim function arguments must be separated by commas and closed with `)`.",
                                    ),
                                );
                            }
                        }
                    }
                }

                self.bump(); // consume `)`

                lhs = Node::Call(Call {
                    callee: Box::new(lhs),
                    args,
                });

                continue;
            }

            if self.peek_kind() == TokenKind::LParen {
                const CALL_BP: u8 = 95;

                if CALL_BP < min_bp {
                    break;
                }

                lhs = self.parse_call_suffix(lhs)?;
                continue;
            }

            let op = self.peek_kind();

            let Some((l_bp, r_bp, infix_kind)) = infix_binding_power(op) else {
                break;
            };

            if l_bp < min_bp {
                break;
            }

            self.bump();

            let rhs = self.parse_bp(r_bp)?;
            lhs = build_infix(infix_kind, lhs, rhs);
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

    fn parse_call_suffix(&mut self, callee: Node) -> Result<Node, Diagnostic> {
        self.bump(); // consume `(`

        let mut args = Vec::new();

        if self.peek_kind() != TokenKind::RParen {
            loop {
                args.push(self.parse_expr()?);

                match self.peek_kind() {
                    TokenKind::Comma => {
                        self.bump();
                    }

                    TokenKind::RParen => break,

                    _ => {
                        return Err(
                            Diagnostic::error(
                                "invalid function call",
                                self.current_span(),
                            )
                            .with_help(
                                "Druim function arguments must be separated by commas and closed with `)`.",
                            ),
                        );
                    }
                }
            }
        }

        self.bump(); // consume `)`

        Ok(Node::Call(Call {
            callee: Box::new(callee),
            args,
        }))
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
    }
}
