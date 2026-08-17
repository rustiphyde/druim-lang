use crate::compiler::ast::{
    BagEntry, BagLiteral, Bind, Block, BlockSegment, BoxLiteral, Call,
    ConversionType, Convert, Copy, Define, DefineEmpty, Func, Guard,
    GuardBranch, InterpolatedText, Literal, Loop, Mutate, Node, NodeKind,
    Param, Program, Print, Ret, TextPart,
};
use crate::compiler::error::{Span, Diagnostic};
use crate::compiler::token::{Token, TokenKind};

pub struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
    in_block: bool,
    in_func: bool, 
}

#[derive(Debug, Clone, Copy, Default)]
struct StatementModifiers {
    stone: bool,
    scope: StatementScope,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum StatementScope {
    #[default]
    Normal,
    Local,
    Global,
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

    pub fn parse_file(&mut self) -> Result<Program, Diagnostic> {
        // A complete .drm file must begin with the Druim file boundary.
        if self.peek_kind() != TokenKind::ProgramBoundary {
            return Err(
                Diagnostic::error(
                    "missing Druim file opening boundary",
                    self.current_span(),
                )
                .with_help(
                    "A Druim source file must begin with `:-:-:`.",
                ),
            );
        }

        self.bump(); // consume opening `:-:-:`

        let mut nodes = Vec::new();

        while self.peek_kind() != TokenKind::ProgramBoundary {
            if self.peek_kind() == TokenKind::Eof {
                return Err(
                    Diagnostic::error(
                        "missing Druim file closing boundary",
                        self.current_span(),
                    )
                    .with_help(
                        "A Druim source file must end with `:-:-:`.",
                    ),
                );
            }

            nodes.push(self.parse_node()?);
        }

        self.bump(); // consume closing `:-:-:`

        // The closing file boundary must be the final structural token.
        if self.peek_kind() != TokenKind::Eof {
            return Err(
                Diagnostic::error(
                    "unexpected content after Druim file boundary",
                    self.current_span(),
                )
                .with_help(
                    "Nothing may appear after the closing `:-:-:` boundary.",
                ),
            );
        }

        Ok(Program { nodes })
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

            TokenKind::LoopStart => {
                self.parse_loop()
            }

            TokenKind::KwStone
                if matches!(
                    (
                        self.tokens.get(self.index + 1).map(|token| &token.kind),
                        self.tokens.get(self.index + 2).map(|token| &token.kind),
                    ),
                    (Some(TokenKind::KwFn), _)
                        | (Some(TokenKind::KwLoc), Some(TokenKind::KwFn))
                        | (Some(TokenKind::KwGlo), Some(TokenKind::KwFn))
                ) =>
            {
                Err(
                    Diagnostic::error(
                        "`stone` cannot modify a function definition",
                        self.current_span(),
                    ),
                )
            }

            TokenKind::KwFn => {
                // parse_func handles:
                // - full function structure validation
                // - parameter rules
                // - body parsing
                self.parse_func()
            }

            TokenKind::KwLoc
                if matches!(
                    self.tokens.get(self.index + 1).map(|token| &token.kind),
                    Some(TokenKind::KwFn)
                ) =>
            {
                let start = self.current_span().start;

                self.bump(); // consume `loc`

                let func = self.parse_func()?;
                let end = func.span.end;

                Ok(Node::new(
                    NodeKind::Local(Box::new(func)),
                    Span {
                        start,
                        end,
                    },
                ))
            }

            TokenKind::KwGlo
                if matches!(
                    self.tokens.get(self.index + 1).map(|token| &token.kind),
                    Some(TokenKind::KwFn)
                ) =>
            {
                let start = self.current_span().start;

                self.bump(); // consume `glo`

                let func = self.parse_func()?;
                let end = func.span.end;

                Ok(Node::new(
                    NodeKind::Global(Box::new(func)),
                    Span {
                        start,
                        end,
                    },
                ))
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

                TokenKind::Print => {
                    return self.parse_print();
                }

                // statement-defining operators
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Mutate
                | TokenKind::Guard => {
                    // DO NOT consume here
                    return match tok.kind {
                        TokenKind::Define      => self.parse_define(),
                        TokenKind::DefineEmpty => self.parse_define_empty(),
                        TokenKind::Copy        => self.parse_copy(),
                        TokenKind::Bind        => self.parse_bind(),
                        TokenKind::Mutate      => self.parse_mutate(),
                        TokenKind::Guard       => self.parse_guard(),
                        _ => unreachable!(),
                    };
                }

                // hard stop: statement boundary
                TokenKind::Semicolon
                | TokenKind::BlockEnd
                | TokenKind::FuncEnd
                | TokenKind::LoopSplit
                | TokenKind::LoopEnd => break,

                _ => i += 1,
            }
        }

        // no statement operator claimed it
        self.parse_call_statement()
    }

    fn parse_statement_modifiers(&mut self) -> Result<StatementModifiers, Diagnostic> {
        let mut modifiers = StatementModifiers::default();

        if self.peek_kind() == TokenKind::KwStone {
            self.bump();
            modifiers.stone = true;

            if self.peek_kind() == TokenKind::KwStone {
                return Err(
                    Diagnostic::error(
                        "repeated `stone` modifier",
                        self.current_span(),
                    )
                    .with_help("`stone` may appear at most once."),
                );
            }
        }

        match self.peek_kind() {
            TokenKind::KwLoc => {
                self.bump();
                modifiers.scope = StatementScope::Local;
            }

            TokenKind::KwGlo => {
                self.bump();
                modifiers.scope = StatementScope::Global;
            }

            _ => {}
        }

        match self.peek_kind() {
            TokenKind::KwLoc | TokenKind::KwGlo | TokenKind::KwStone => {
                return Err(
                    Diagnostic::error(
                        "invalid statement modifier order",
                        self.current_span(),
                    )
                    .with_help(
                        "Druim statement modifiers must use the form:\n\
                        `[stone] [loc | glo] statement`",
                    ),
                );
            }

            _ => {}
        }

        Ok(modifiers)
    }

    fn parse_ret(&mut self) -> Result<Node, Diagnostic> {
        // We are committing to parsing a return statement
         if !self.in_func {
            return Err(
                Diagnostic::error(
                    "return statement outside function",
                    self.current_span(),
                )
                .with_help("`ret` may only appear inside a function body."),
            );
        }

        // We are committing to parsing a return statement
        let start = self.current_span().start;
        self.bump(); // consume `ret`

        // 🔒 REQUIRED: verify semicolon exists BEFORE parsing anything else
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                let end = self
                    .tokens
                    .last()
                    .map(|token| token.pos + token.lexeme.len())
                    .unwrap_or(0);

                return Err(
                    Diagnostic::error(
                        "unterminated return statement",
                        Span {
                            start: end,
                            end,
                        },
                    )
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
            let semicolon = self
                .bump()
                .expect("semicolon token must exist");

            return Ok(Node::new(
                NodeKind::Ret(Ret { value: None }),
                Span {
                    start,
                    end: semicolon.pos + semicolon.lexeme.len(),
                },
            ));
        }

        // Disallow statement operators inside return value
        let mut i = self.index;
        while i < stmt_end {
            match self.tokens[i].kind {
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Mutate
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
                Node::new(
                    NodeKind::Ident(ident.lexeme.clone()),
                    Span {
                        start: ident.pos,
                        end: ident.pos + ident.lexeme.len(),
                    },
                )
            } else {
                self.parse_rhs()?
            };

        let semicolon = self
            .bump()
            .expect("terminating semicolon must exist");

        Ok(Node::new(
            NodeKind::Ret(Ret {
                value: Some(Box::new(value)),
            }),
            Span {
                start,
                end: semicolon.pos + semicolon.lexeme.len(),
            },
        ))
    }

    fn parse_print(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

        self.bump(); // consume `|>`

        if self.peek_kind() != TokenKind::LParen {
            return Err(
                Diagnostic::error(
                    "invalid print statement",
                    self.current_span(),
                )
                .with_help(
                    "Druim Print requires `(` after `|>`.\n\
                    Example: `|> (\"Hello World!\");`",
                ),
            );
        }

        self.bump(); // consume `(`

        if self.peek_kind() == TokenKind::RParen {
            return Err(
                Diagnostic::error(
                    "empty print statement",
                    self.current_span(),
                )
                .with_help(
                    "Druim Print requires a value inside `(` and `)`.\n\
                    Example: `|> (\"Hello World!\");`",
                ),
            );
        }

        let value = self.parse_expr()?;

        if self.peek_kind() != TokenKind::RParen {
            return Err(
                Diagnostic::error(
                    "invalid print statement",
                    self.current_span(),
                )
                .with_help(
                    "Druim Print accepts one complete expression inside `(` and `)`.",
                ),
            );
        }

        self.bump(); // consume `)`

        if self.peek_kind() != TokenKind::Semicolon {
            return Err(
                Diagnostic::error(
                    "unterminated print statement",
                    self.current_span(),
                )
                .with_help(
                    "Druim expected `;` after the Print statement.\n\
                    Example: `|> (\"Hello World!\");`",
                ),
            );
        }

        let semicolon = self
            .bump()
            .expect("terminating semicolon must exist");

        Ok(Node::new(
            NodeKind::Print(Print {
                value: Box::new(value),
            }),
            Span {
                start,
                end: semicolon.pos + semicolon.lexeme.len(),
            },
        ))
    }

    fn parse_define_empty(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

        let modifiers = self.parse_statement_modifiers()?;

        // Identifier
        let ident_tok = match self.bump() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error(
                        "invalid empty definition",
                        self.current_span(),
                    )
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

        // Consume `=;`
        let operator = self
            .bump()
            .expect("empty define operator must exist");

        let statement_span = Span {
            start,
            end: operator.pos + operator.lexeme.len(),
        };

        // Chaining is illegal
        match self.peek_kind() {
            TokenKind::Define
            | TokenKind::DefineEmpty
            | TokenKind::Copy
            | TokenKind::Bind
            | TokenKind::Mutate
            | TokenKind::Guard => {
                return Err(
                    Diagnostic::error(
                        "invalid empty definition",
                        self.current_span(),
                    )
                    .with_help(
                        "Statement operators cannot be chained.\n\
                        Split this into multiple statements.\n\
                        Example: `a =; b = 1;`",
                    ),
                );
            }
            _ => {}
        }

        let node = Node::new(
            NodeKind::DefineEmpty(DefineEmpty { name }),
            statement_span,
        );

        let node = match modifiers.scope {
            StatementScope::Normal => node,

            StatementScope::Local => Node::new(
                NodeKind::Local(Box::new(node)),
                statement_span,
            ),

            StatementScope::Global => Node::new(
                NodeKind::Global(Box::new(node)),
                statement_span,
            ),
        };

        if modifiers.stone {
            Ok(Node::new(
                NodeKind::Stone(Box::new(node)),
                statement_span,
            ))
        } else {
            Ok(node)
        }
    }

    fn parse_define(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

        // Statement MUST terminate
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                let end = self
                    .tokens
                    .last()
                    .map(|token| token.pos + token.lexeme.len())
                    .unwrap_or(0);

                return Err(
                    Diagnostic::error(
                        "unterminated define statement",
                        Span {
                            start: end,
                            end,
                        },
                    )
                    .with_help(
                        "Druim expected a semicolon `;` to terminate this define statement.\n\
                        Example: `x = 42;`",
                    ),
                );
            }
        };

        let modifiers = self.parse_statement_modifiers()?;

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
                | TokenKind::Mutate
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

        let semicolon = self
            .bump()
            .expect("terminating semicolon must exist");

        let statement_span = Span {
            start,
            end: semicolon.pos + semicolon.lexeme.len(),
        };

        let node = Node::new(
            NodeKind::Define(Define {
                name,
                value: Box::new(value),
            }),
            statement_span,
        );

        let node = match modifiers.scope {
            StatementScope::Normal => node,

            StatementScope::Local => Node::new(
                NodeKind::Local(Box::new(node)),
                statement_span,
            ),

            StatementScope::Global => Node::new(
                NodeKind::Global(Box::new(node)),
                statement_span,
            ),
        };

        if modifiers.stone {
            Ok(Node::new(
                NodeKind::Stone(Box::new(node)),
                statement_span,
            ))
        } else {
            Ok(node)
        }
    }

    fn parse_copy(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

        match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                let end = self
                    .tokens
                    .last()
                    .map(|token| token.pos + token.lexeme.len())
                    .unwrap_or(0);

                return Err(
                    Diagnostic::error(
                        "unterminated copy statement",
                        Span {
                            start: end,
                            end,
                        },
                    )
                    .with_help(
                        "Druim expected a semicolon `;` to terminate this copy statement.\n\
                        Example: `a := b;`",
                    ),
                );
            }
        };

        let modifiers = self.parse_statement_modifiers()?;

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
                    | TokenKind::Mutate
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
        let semicolon = self
            .bump()
            .expect("terminating semicolon must exist");

        let statement_span = Span {
            start,
            end: semicolon.pos + semicolon.lexeme.len(),
        };

        let node = Node::new(
            NodeKind::Copy(Copy { name, target }),
            statement_span,
        );

        let node = match modifiers.scope {
            StatementScope::Normal => node,

            StatementScope::Local => Node::new(
                NodeKind::Local(Box::new(node)),
                statement_span,
            ),

            StatementScope::Global => Node::new(
                NodeKind::Global(Box::new(node)),
                statement_span,
            ),
        };

        if modifiers.stone {
            Ok(Node::new(
                NodeKind::Stone(Box::new(node)),
                statement_span,
            ))
        } else {
            Ok(node)
        }
    }

    fn parse_bind(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

        match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(_) => {}
            None => {
                let end = self
                    .tokens
                    .last()
                    .map(|token| token.pos + token.lexeme.len())
                    .unwrap_or(0);

                return Err(
                    Diagnostic::error(
                        "unterminated bind statement",
                        Span {
                            start: end,
                            end,
                        },
                    )
                    .with_help(
                        "Druim expected a semicolon `;` to terminate this bind statement.\n\
                        Example: `a :> b;`",
                    ),
                );
            }
        }

        let modifiers = self.parse_statement_modifiers()?;

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
                    | TokenKind::Mutate
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
        let semicolon = self
            .bump()
            .expect("terminating semicolon must exist");

        let statement_span = Span {
            start,
            end: semicolon.pos + semicolon.lexeme.len(),
        };

        let node = Node::new(
            NodeKind::Bind(Bind { name, target }),
            statement_span,
        );

        let node = match modifiers.scope {
            StatementScope::Normal => node,

            StatementScope::Local => Node::new(
                NodeKind::Local(Box::new(node)),
                statement_span,
            ),

            StatementScope::Global => Node::new(
                NodeKind::Global(Box::new(node)),
                statement_span,
            ),
        };

        if modifiers.stone {
            Ok(Node::new(
                NodeKind::Stone(Box::new(node)),
                statement_span,
            ))
        } else {
            Ok(node)
        }
    }

    fn parse_mutate(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                let end = self
                    .tokens
                    .last()
                    .map(|token| token.pos + token.lexeme.len())
                    .unwrap_or(0);

                return Err(
                    Diagnostic::error(
                        "unterminated mutate statement",
                        Span {
                            start: end,
                            end,
                        },
                    )
                    .with_help(
                        "Druim expected a semicolon `;` to terminate this mutate statement.\n\
                        Example: `count << count + 1;`",
                    ),
                );
            }
        };

        let modifiers = self.parse_statement_modifiers()?;

        // Left-hand identifier
        let ident_tok = match self.bump() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error(
                        "invalid mutate statement",
                        self.current_span(),
                    )
                    .with_help(
                        "Druim mutate statements must begin with an identifier.\n\
                        Example: `count << count + 1;`",
                    ),
                );
            }
        };

        if ident_tok.kind != TokenKind::Ident {
            return Err(
                Diagnostic::error(
                    "invalid mutate statement",
                    Span {
                        start: ident_tok.pos,
                        end: ident_tok.pos + ident_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "Druim mutate statements must begin with an identifier.\n\
                    Example: `count << count + 1;`",
                ),
            );
        }

        let name = ident_tok.lexeme.clone();

        // Consume `<<`
        self.bump();

        // RHS must exist
        if self.peek_kind() == TokenKind::Semicolon {
            return Err(
                Diagnostic::error(
                    "invalid mutate statement",
                    self.current_span(),
                )
                .with_help(
                    "A mutate statement requires a value after `<<`.\n\
                    Example: `count << count + 1;`",
                ),
            );
        }

        // Statement operators cannot appear inside the RHS.
        let mut i = self.index;

        while i < stmt_end {
            match self.tokens[i].kind {
                TokenKind::Define
                | TokenKind::DefineEmpty
                | TokenKind::Copy
                | TokenKind::Bind
                | TokenKind::Mutate
                | TokenKind::Guard => {
                    return Err(
                        Diagnostic::error(
                            "invalid mutate statement",
                            Span {
                                start: self.tokens[i].pos,
                                end: self.tokens[i].pos + self.tokens[i].lexeme.len(),
                            },
                        )
                        .with_help(
                            "Mutate statements cannot contain other statement operators.\n\
                            Split this into separate statements.",
                        ),
                    );
                }

                _ => {}
            }

            i += 1;
        }

        // Mutate accepts one complete expression.
        let value = self.parse_expr()?;

        // The complete RHS must be consumed.
        let next_tok = match self.peek() {
            Some(tok) => tok,
            None => {
                return Err(
                    Diagnostic::error(
                        "unterminated mutate statement",
                        self.current_span(),
                    )
                    .with_help(
                        "Druim expected a semicolon `;` after the mutated value.\n\
                        Example: `count << count + 1;`",
                    ),
                );
            }
        };

        if next_tok.kind != TokenKind::Semicolon {
            return Err(
                Diagnostic::error(
                    "invalid mutate statement",
                    Span {
                        start: next_tok.pos,
                        end: next_tok.pos + next_tok.lexeme.len(),
                    },
                )
                .with_help(
                    "A Druim mutate statement must contain exactly one complete expression.\n\
                    Unexpected tokens remain after the mutated value.\n\
                    Example: `count << count + 1;`",
                ),
            );
        }

        let semicolon = self
            .bump()
            .expect("terminating semicolon must exist");

        let statement_span = Span {
            start,
            end: semicolon.pos + semicolon.lexeme.len(),
        };

        let node = Node::new(
            NodeKind::Mutate(Mutate {
                name,
                value: Box::new(value),
            }),
            statement_span,
        );

        let node = match modifiers.scope {
            StatementScope::Normal => node,

            StatementScope::Local => Node::new(
                NodeKind::Local(Box::new(node)),
                statement_span,
            ),

            StatementScope::Global => Node::new(
                NodeKind::Global(Box::new(node)),
                statement_span,
            ),
        };

        if modifiers.stone {
            Ok(Node::new(
                NodeKind::Stone(Box::new(node)),
                statement_span,
            ))
        } else {
            Ok(node)
        }
    }

    fn parse_guard(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

        // Find statement terminator FIRST
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                let end = self
                    .tokens
                    .last()
                    .map(|token| token.pos + token.lexeme.len())
                    .unwrap_or(0);

                return Err(
                    Diagnostic::error(
                        "unterminated guard statement",
                        Span {
                            start: end,
                            end,
                        },
                    )
                    .with_help(
                        "Druim expected a semicolon `;` to terminate this guard statement.\n\
                        Example: `x ?= y;`",
                    ),
                );
            }
        };

        let modifiers = self.parse_statement_modifiers()?;

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
                | TokenKind::Mutate
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
        let semicolon = self
            .bump()
            .expect("terminating semicolon must exist");

        let statement_span = Span {
            start,
            end: semicolon.pos + semicolon.lexeme.len(),
        };

        let node = Node::new(
            NodeKind::Guard(Guard {
                target: name,
                branches,
            }),
            statement_span,
        );

        let node = match modifiers.scope {
            StatementScope::Normal => node,

            StatementScope::Local => Node::new(
                NodeKind::Local(Box::new(node)),
                statement_span,
            ),

            StatementScope::Global => Node::new(
                NodeKind::Global(Box::new(node)),
                statement_span,
            ),
        };

        if modifiers.stone {
            Ok(Node::new(
                NodeKind::Stone(Box::new(node)),
                statement_span,
            ))
        } else {
            Ok(node)
        }
    }

    fn parse_block(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

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
            let end = self
                .tokens
                .last()
                .map(|token| token.pos + token.lexeme.len())
                .unwrap_or(0);

            return Err(
                Diagnostic::error(
                    "unterminated block structure",
                    Span {
                        start: end,
                        end,
                    },
                )
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

            nodes.push(self.parse_node()?);
        }

        // Store the final segment
        segments.push(BlockSegment { nodes });

        // Consume closing delimiter
        let block_end = self
            .bump()
            .cloned()
            .expect("closing block delimiter must exist");

        // Exit block context
        self.in_block = prev;

        Ok(Node::new(
            NodeKind::Block(Block { segments }),
            Span {
                start,
                end: block_end.pos + block_end.lexeme.len(),
            },
        ))
    }

    fn parse_loop(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

        self.bump(); // consume `:<`

        let mut setup = Vec::new();

        // Parse setup statements until the first `>?<`.
        while self.peek_kind() != TokenKind::LoopSplit {
            match self.peek_kind() {
                TokenKind::Eof | TokenKind::LoopEnd => {
                    return Err(
                        Diagnostic::error(
                            "incomplete loop structure",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected the first loop separator `>?<` after the setup section.\n\
                            Loop structure:\n\
                            `:< setup >?< condition >?< process >:`",
                        ),
                    );
                }

                TokenKind::LoopStart => {
                    setup.push(self.parse_loop()?);
                }

                _ => {
                    setup.push(self.parse_statement_entry()?);
                }
            }
        }

        self.bump(); // consume first `>?<`

        // The condition is required.
        if self.peek_kind() == TokenKind::LoopSplit {
            return Err(
                Diagnostic::error(
                    "missing loop condition",
                    self.current_span(),
                )
                .with_help(
                    "Druim loops require one condition expression between the two `>?<` separators.",
                ),
            );
        }

        if matches!(
            self.peek_kind(),
            TokenKind::LoopEnd | TokenKind::Eof
        ) {
            return Err(
                Diagnostic::error(
                    "incomplete loop structure",
                    self.current_span(),
                )
                .with_help(
                    "Druim expected a condition followed by the second loop separator `>?<`.",
                ),
            );
        }

        let condition = self.parse_expr()?;

        // The condition must consume everything up to the second separator.
        if self.peek_kind() != TokenKind::LoopSplit {
            return Err(
                Diagnostic::error(
                    "invalid loop condition",
                    self.current_span(),
                )
                .with_help(
                    "A Druim loop condition must contain exactly one complete expression followed by `>?<`.",
                ),
            );
        }

        self.bump(); // consume second `>?<`

        let mut process = Vec::new();

        // Parse process statements until `>:`.
        while self.peek_kind() != TokenKind::LoopEnd {
            match self.peek_kind() {
                TokenKind::Eof => {
                    return Err(
                        Diagnostic::error(
                            "unterminated loop structure",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected a closing loop delimiter `>:`.",
                        ),
                    );
                }

                TokenKind::LoopSplit => {
                    return Err(
                        Diagnostic::error(
                            "too many loop separators",
                            self.current_span(),
                        )
                        .with_help(
                            "A Druim loop requires exactly two `>?<` separators.",
                        ),
                    );
                }

                TokenKind::LoopStart => {
                    process.push(self.parse_loop()?);
                }

                _ => {
                    process.push(self.parse_statement_entry()?);
                }
            }
        }

        let loop_end = self
            .bump()
            .expect("closing loop delimiter must exist");

        Ok(Node::new(
            NodeKind::Loop(Loop {
                setup,
                condition: Box::new(condition),
                process,
            }),
            Span {
                start,
                end: loop_end.pos + loop_end.lexeme.len(),
            },
        ))
    }

    fn parse_func(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

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

            if !self.tokens[self.index..]
                .iter()
                .any(|t| t.kind == TokenKind::FuncEnd)
            {
                let end = self
                    .tokens
                    .last()
                    .map(|token| token.pos + token.lexeme.len())
                    .unwrap_or(0);

                return Err(
                    Diagnostic::error(
                        "unterminated function structure",
                        Span {
                            start: end,
                            end,
                        },
                    )
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
                let func_end = &self.tokens[i];

                return Err(
                    Diagnostic::error(
                        "incomplete function definition",
                        Span {
                            start: func_end.pos,
                            end: func_end.pos + func_end.lexeme.len(),
                        },
                    )
                    .with_help(
                        "Druim functions must consist of a parameter list and at least one body.\n\
                        An empty list and empty body is allowed, but a body delimiter `)(` is required.\n\
                        Example: `fn f :()():`",
                    ),
                );
            }

            // Parse parameters
            let mut params = Vec::new();
            let mut param_names = std::collections::HashSet::new();

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

                    if !param_names.insert(param_name.clone()) {
                        return Err(
                            Diagnostic::error(
                                "duplicate function parameter",
                                Span {
                                    start: ident_tok.pos,
                                    end: ident_tok.pos + ident_tok.lexeme.len(),
                                },
                            )
                            .with_help(
                                "Druim function parameter names must be unique within the same parameter list.",
                            ),
                        );
                    }

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
                nodes.push(self.parse_node()?);
            }

            let func_end = self
                .bump()
                .expect("closing function delimiter must exist");

            Ok(Node::new(
                NodeKind::Func(Func {
                    name,
                    params,
                    body: nodes,
                }),
                Span {
                    start,
                    end: func_end.pos + func_end.lexeme.len(),
                },
            ))
        })();

        self.in_func = prev_in_func;
        result
    }

    fn parse_rhs(&mut self) -> Result<Node, Diagnostic> {
        let value = self.parse_expr()?;

        // Bare identifiers are not values
        if matches!(&value.kind, NodeKind::Ident(_)) {
            return Err(
                Diagnostic::error("invalid value expression", value.span)
                    .with_help(
                        "A bare identifier is not a value.\n\
                        Use a function call, copy (`:=`), or bind (`:>`) instead.",
                    ),
            );
        }

        Ok(value)
    }

    fn parse_call_statement(&mut self) -> Result<Node, Diagnostic> {
        let start = self.current_span().start;

        // Verify statement terminates
        let stmt_end = match self.tokens[self.index..]
            .iter()
            .position(|t| t.kind == TokenKind::Semicolon)
        {
            Some(off) => self.index + off,
            None => {
                let end = self
                    .tokens
                    .last()
                    .map(|token| token.pos + token.lexeme.len())
                    .unwrap_or(0);

                return Err(
                    Diagnostic::error(
                        "unterminated function call statement",
                        Span {
                            start: end,
                            end,
                        },
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
                | TokenKind::Mutate
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
        let mut call = self.parse_expr()?;

        // A standalone expression must structurally be a function call
        if !matches!(&call.kind, NodeKind::Call(_)) {
            return Err(
                Diagnostic::error(
                    "invalid function call statement",
                    call.span,
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

        let semicolon = self
            .bump()
            .expect("terminating semicolon must exist");

        call.span = Span {
            start,
            end: semicolon.pos + semicolon.lexeme.len(),
        };

        Ok(call)
    }

    pub fn parse_expr(&mut self) -> Result<Node, Diagnostic> {
        self.parse_bp(0)
    }

    // ===== Pratt parser =====

    fn parse_bp(&mut self, min_bp: u8) -> Result<Node, Diagnostic> {
        let mut lhs = self.parse_prefix()?;

        loop {
            const CALL_BP: u8 = 95;

            // Postfix function call: lhs(...)
            if self.peek_kind() == TokenKind::LParen {
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

            if matches!(&lhs.kind, NodeKind::Has(_, _))
                && matches!(infix_kind, Infix::Get | Infix::Has)
            {
                 return Err(
                    Diagnostic::error(
                        "Druim cannot continue traversal after `:?`",
                        self.current_span(),
                    )
                    .with_help("`:?` terminates a Druim traversal chain. Use `::` before the final `:?`."),
                );
            }

            self.bump(); // consume infix operator

            let rhs = if matches!(infix_kind, Infix::Get | Infix::Has)
                && self.peek_kind() == TokenKind::LBracket
            {
                let bracket_start = self
                    .bump()
                    .cloned()
                    .expect("opening index bracket must exist");

                let index = self.parse_bp(0)?;

                if self.peek_kind() != TokenKind::RBracket {
                    return Err(
                        Diagnostic::error(
                            "expected `]` after Box index",
                            self.current_span(),
                        )
                        .with_help("Close the indexed selector with `]`."),
                    );
                }

                let bracket_end = self
                    .bump()
                    .cloned()
                    .expect("closing index bracket must exist");

                Node::new(
                    NodeKind::Index(Box::new(index)),
                    Span {
                        start: bracket_start.pos,
                        end: bracket_end.pos + bracket_end.lexeme.len(),
                    },
                )
            } else {
                self.parse_bp(r_bp)?
            };

            lhs = build_infix(infix_kind, lhs, rhs);
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Node, Diagnostic> {
        let span_start = self.current_span().start;

        let tok = self.bump().cloned().ok_or_else(|| {
            Diagnostic::error(
                "unexpected end of input",
                Span {
                    start: span_start,
                    end: span_start,
                },
            )
            .with_help("Druim expected a value expression here.")
        })?;

        let token_span = Span {
            start: tok.pos,
            end: tok.pos + tok.lexeme.len(),
        };

        match tok.kind {
            // ─── Atoms ──────────────────────────────
            TokenKind::Ident => Ok(Node::new(
                NodeKind::Ident(tok.lexeme.clone()),
                token_span,
            )),

            TokenKind::NumLit => {
                let n = tok.lexeme.parse::<i64>().unwrap_or(0);

                Ok(Node::new(
                    NodeKind::Lit(Literal::Num(n)),
                    token_span,
                ))
            }

            TokenKind::DecLit => Ok(Node::new(
                NodeKind::Lit(Literal::Dec(tok.lexeme.clone())),
                token_span,
            )),

            TokenKind::TextLit => Ok(Node::new(
                NodeKind::Lit(Literal::Text(tok.lexeme.clone())),
                token_span,
            )),

            TokenKind::TextStart => {
                return self.parse_interpolated_text(tok.pos);
            }

            TokenKind::FlagLit => {
                let value = match tok.lexeme.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => {
                        return Err(
                            Diagnostic::error("invalid flag literal", token_span)
                                .with_help(
                                    "Druim flag literals must be either `true` or `false`.",
                                ),
                        );
                    }
                };

                Ok(Node::new(
                    NodeKind::Lit(Literal::Flag(value)),
                    token_span,
                ))
            }

            TokenKind::KwVoid => {
                if self.peek_kind() == TokenKind::LParen {
                    return Err(
                        Diagnostic::error(
                            "`void` is not a conversion expression",
                            Span {
                                start: tok.pos,
                                end: self.current_span().end,
                            },
                        )
                        .with_help(
                            "`void` is Druim's explicit absence literal and is not callable.\n\
                            Use `void` directly.",
                        ),
                    );
                }

                Ok(Node::new(
                    NodeKind::Lit(Literal::Void),
                    token_span,
                ))
            }

            // ─── Type conversions ──────────────────
            TokenKind::KwNum => {
                self.parse_conversion(tok.pos, ConversionType::Num)
            }

            TokenKind::KwDec => {
                self.parse_conversion(tok.pos, ConversionType::Dec)
            }

            TokenKind::KwText => {
                self.parse_conversion(tok.pos, ConversionType::Text)
            }

            TokenKind::KwFlag => {
                self.parse_conversion(tok.pos, ConversionType::Flag)
            }

            // ─── Collection literals ───────────────
            TokenKind::BoxStart => self.parse_box_literal(tok.pos),

            TokenKind::BagStart => self.parse_bag_literal(tok.pos),

            // ─── Unary operators ────────────────────
            TokenKind::Not => {
                let rhs = self.parse_bp(PREFIX_BP)?;

                let span = Span {
                    start: tok.pos,
                    end: rhs.span.end,
                };

                Ok(Node::new(
                    NodeKind::Not(Box::new(rhs)),
                    span,
                ))
            }

            TokenKind::Sub => {
                let rhs = self.parse_bp(PREFIX_BP)?;

                let span = Span {
                    start: tok.pos,
                    end: rhs.span.end,
                };

                Ok(Node::new(
                    NodeKind::Neg(Box::new(rhs)),
                    span,
                ))
            }

            // ─── Grouping ───────────────────────────
            TokenKind::LParen => {
                let expr = self.parse_bp(0)?;

                let closing_span = self.current_span();
                self.expect(TokenKind::RParen, "`)`")?;

                if !is_math_expression(&expr) {
                    return Err(
                        Diagnostic::error(
                            "invalid parenthesized expression",
                            Span {
                                start: tok.pos,
                                end: closing_span.end,
                            },
                        )
                        .with_help(
                            "Parentheses may only group mathematical expressions.\n\
                            They are not general-purpose value delimiters.",
                        ),
                    );
                }

                Ok(expr)
            }

            // ─── Explicitly illegal value starters ──
            TokenKind::Define
            | TokenKind::DefineEmpty
            | TokenKind::Copy
            | TokenKind::Bind
            | TokenKind::Mutate
            | TokenKind::Guard => {
                Err(
                    Diagnostic::error(
                        "invalid value expression",
                        token_span,
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
            | TokenKind::BlockStart
            | TokenKind::LoopStart => {
                Err(
                    Diagnostic::error(
                        "invalid value expression",
                        token_span,
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
                    token_span,
                )
                .with_help("Druim expected a value here."),
            ),
        }
    }

    fn parse_conversion(
        &mut self,
        start: usize,
        target: ConversionType,
    ) -> Result<Node, Diagnostic> {
        if self.peek_kind() != TokenKind::LParen {
            return Err(
                Diagnostic::error(
                    "invalid type conversion",
                    self.current_span(),
                )
                .with_help(
                    "Druim type conversion requires one expression inside `(` and `)`.\n\
                    Examples: `num(value)`, `dec(value)`, `text(value)`, `flag(value)`",
                ),
            );
        }

        self.bump(); // consume `(`

        if self.peek_kind() == TokenKind::RParen {
            return Err(
                Diagnostic::error(
                    "empty type conversion",
                    self.current_span(),
                )
                .with_help(
                    "Druim type conversion requires exactly one expression.\n\
                    Examples: `num(value)`, `dec(value)`, `text(value)`, `flag(value)`",
                ),
            );
        }

        let value = self.parse_expr()?;

        match self.peek_kind() {
            TokenKind::RParen => {}

            TokenKind::Comma => {
                return Err(
                    Diagnostic::error(
                        "too many type conversion arguments",
                        self.current_span(),
                    )
                    .with_help(
                        "Druim type conversions accept exactly one expression.\n\
                        Examples: `num(value)`, `dec(value)`, `text(value)`, `flag(value)`",
                    ),
                );
            }

            _ => {
                return Err(
                    Diagnostic::error(
                        "invalid type conversion",
                        self.current_span(),
                    )
                    .with_help(
                        "A Druim type conversion must contain exactly one complete expression followed by `)`.",
                    ),
                );
            }
        }

        let closing = self
            .bump()
            .expect("closing conversion parenthesis must exist");

        Ok(Node::new(
            NodeKind::Convert(Convert {
                target,
                value: Box::new(value),
            }),
            Span {
                start,
                end: closing.pos + closing.lexeme.len(),
            },
        ))
    }

    fn parse_interpolated_text(
        &mut self,
        start: usize,
    ) -> Result<Node, Diagnostic> {
        let mut parts = Vec::new();

        while self.peek_kind() != TokenKind::TextEnd {
            match self.peek_kind() {
                TokenKind::TextLit => {
                    let token = self
                        .bump()
                        .expect("TextLit token must exist");

                    parts.push(TextPart::Text(
                        token.lexeme.clone(),
                    ));
                }

                TokenKind::InterpStart => {
                    self.bump(); // consume `:.`

                    if self.peek_kind() == TokenKind::InterpEnd {
                        return Err(
                            Diagnostic::error(
                                "empty text interpolation",
                                self.current_span(),
                            )
                            .with_help(
                                "Text interpolation requires a Druim expression between `:.` and `.:`.",
                            ),
                        );
                    }

                    let expr = self.parse_expr()?;

                    if self.peek_kind() != TokenKind::InterpEnd {
                        return Err(
                            Diagnostic::error(
                                "invalid text interpolation",
                                self.current_span(),
                            )
                            .with_help(
                                "Druim expected `.:` after the interpolated expression.",
                            ),
                        );
                    }

                    self.bump(); // consume `.:`

                    parts.push(TextPart::Expr(expr));
                }

                TokenKind::Eof => {
                    return Err(
                        Diagnostic::error(
                            "unterminated interpolated text",
                            self.current_span(),
                        ),
                    );
                }

                _ => {
                    return Err(
                        Diagnostic::error(
                            "invalid token inside interpolated text",
                            self.current_span(),
                        ),
                    );
                }
            }
        }

        let end_token = self
            .bump()
            .expect("TextEnd token must exist");

        Ok(Node::new(
            NodeKind::InterpolatedText(
                InterpolatedText { parts }
            ),
            Span {
                start,
                end: end_token.pos + end_token.lexeme.len(),
            },
        ))
    }

    fn parse_box_literal(&mut self, start: usize) -> Result<Node, Diagnostic> {
        let mut values = Vec::new();

        // Empty Box: `:[]:`
        if self.peek_kind() == TokenKind::BoxEnd {
            let box_end = self
                .bump()
                .expect("closing Box delimiter must exist");

            return Ok(Node::new(
                NodeKind::Box(BoxLiteral { values }),
                Span {
                    start,
                    end: box_end.pos + box_end.lexeme.len(),
                },
            ));
        }

        loop {
            // A value must appear here.
            match self.peek_kind() {
                TokenKind::Comma => {
                    return Err(
                        Diagnostic::error(
                            "missing Box value",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected a Box value before this comma.\n\
                            Example: `:[1, 2, 3]:`",
                        ),
                    );
                }

                TokenKind::Semicolon => {
                    return Err(
                        Diagnostic::error(
                            "invalid separator in Box literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Box values must be separated by commas, not semicolons.\n\
                            Example: `:[1, 2, 3]:`",
                        ),
                    );
                }

                TokenKind::BoxEnd => {
                    return Err(
                        Diagnostic::error(
                            "missing Box value",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected a value after the previous comma.\n\
                            Trailing commas are not allowed in Box literals.",
                        ),
                    );
                }

                TokenKind::Eof => {
                    return Err(
                        Diagnostic::error(
                            "unterminated Box literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected a closing Box delimiter `]:`.\n\
                            Example: `:[1, 2, 3]:`",
                        ),
                    );
                }

                _ => {}
            }

            values.push(self.parse_expr()?);

            // After a complete value, only `,` or `]:` is valid.
            match self.peek_kind() {
                TokenKind::Comma => {
                    self.bump(); // consume `,`

                    // Reject trailing comma immediately.
                    if self.peek_kind() == TokenKind::BoxEnd {
                        return Err(
                            Diagnostic::error(
                                "trailing comma in Box literal",
                                self.current_span(),
                            )
                            .with_help(
                                "Druim Box literals do not allow trailing commas.\n\
                                Remove the comma before `]:`.",
                            ),
                        );
                    }
                }

                TokenKind::BoxEnd => {
                    let box_end = self
                        .bump()
                        .expect("closing Box delimiter must exist");

                    return Ok(Node::new(
                        NodeKind::Box(BoxLiteral { values }),
                        Span {
                            start,
                            end: box_end.pos + box_end.lexeme.len(),
                        },
                    ));
                }

                TokenKind::Semicolon => {
                    return Err(
                        Diagnostic::error(
                            "invalid separator in Box literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim Box values must be separated by commas, not semicolons.\n\
                            Example: `:[1, 2, 3]:`",
                        ),
                    );
                }

                TokenKind::Eof => {
                    return Err(
                        Diagnostic::error(
                            "unterminated Box literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected a closing Box delimiter `]:`.",
                        ),
                    );
                }

                _ => {
                    return Err(
                        Diagnostic::error(
                            "missing comma in Box literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim Box values must be separated by commas.\n\
                            Example: `:[1, 2, 3]:`",
                        ),
                    );
                }
            }
        }
    }

    fn parse_bag_literal(&mut self, start: usize) -> Result<Node, Diagnostic> {
        use std::collections::HashSet;

        let mut entries = Vec::new();
        let mut names = HashSet::new();

        // Empty Bag: `:||:`
        if self.peek_kind() == TokenKind::BagEnd {
            let bag_end = self
                .bump()
                .expect("closing Bag delimiter must exist");

            return Ok(Node::new(
                NodeKind::Bag(BagLiteral { entries }),
                Span {
                    start,
                    end: bag_end.pos + bag_end.lexeme.len(),
                },
            ));
        }

        loop {
            // An entry name must appear here.
            match self.peek_kind() {
                TokenKind::Comma => {
                    return Err(
                        Diagnostic::error(
                            "missing Bag entry",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected a named Bag entry before this comma.\n\
                            Example: `:| name: \"Rusty\", level: 42 |:`",
                        ),
                    );
                }

                TokenKind::Semicolon => {
                    return Err(
                        Diagnostic::error(
                            "invalid separator in Bag literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim Bag entries must be separated by commas, not semicolons.\n\
                            Example: `:| name: \"Rusty\", level: 42 |:`",
                        ),
                    );
                }

                TokenKind::BagEnd => {
                    return Err(
                        Diagnostic::error(
                            "missing Bag entry",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected an entry after the previous comma.\n\
                            Trailing commas are not allowed in Druim Bag literals.",
                        ),
                    );
                }

                TokenKind::Eof => {
                    return Err(
                        Diagnostic::error(
                            "unterminated Bag literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected a closing Bag delimiter `|:`.\n\
                            Example: `:| name: \"Rusty\" |:`",
                        ),
                    );
                }

                _ => {}
            }

            let name_tok = match self.bump() {
                Some(tok) => tok.clone(),

                None => {
                    return Err(
                        Diagnostic::error(
                            "unterminated Bag literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected a named Bag entry or the closing delimiter `|:`.",
                        ),
                    );
                }
            };

            if name_tok.kind != TokenKind::Ident {
                return Err(
                    Diagnostic::error(
                        "invalid Bag entry name",
                        Span {
                            start: name_tok.pos,
                            end: name_tok.pos + name_tok.lexeme.len(),
                        },
                    )
                    .with_help(
                        "Each Druim Bag entry must begin with an identifier followed by `:`.\n\
                        Example: `name: \"Rusty\"`",
                    ),
                );
            }

            let name = name_tok.lexeme.clone();

            if !names.insert(name.clone()) {
                return Err(
                    Diagnostic::error(
                        "duplicate Bag entry name",
                        Span {
                            start: name_tok.pos,
                            end: name_tok.pos + name_tok.lexeme.len(),
                        },
                    )
                    .with_help(
                        "Druim Bag entry names must be unique within the same Bag literal.",
                    ),
                );
            }

            self.expect(
                TokenKind::Colon,
                "Druim expected `:` after the Bag entry name.",
            )?;

            // An entry value must appear here.
            match self.peek_kind() {
                TokenKind::Comma | TokenKind::BagEnd | TokenKind::Eof => {
                    return Err(
                        Diagnostic::error(
                            "missing Bag entry value",
                            self.current_span(),
                        )
                        .with_help(
                            "Each Druim Bag entry requires a value after `:`.\n\
                            Example: `name: \"Rusty\"`",
                        ),
                    );
                }

                TokenKind::Semicolon => {
                    return Err(
                        Diagnostic::error(
                            "invalid separator in Bag literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim Bag entries must be separated by commas, not semicolons.",
                        ),
                    );
                }

                _ => {}
            }

            let value = self.parse_expr()?;

            entries.push(BagEntry { name, value });

            // After a complete entry, only `,` or `|:` is valid.
            match self.peek_kind() {
                TokenKind::Comma => {
                    self.bump(); // consume `,`

                    if self.peek_kind() == TokenKind::BagEnd {
                        return Err(
                            Diagnostic::error(
                                "trailing comma in Bag literal",
                                self.current_span(),
                            )
                            .with_help(
                                "Druim Bag literals do not allow trailing commas.\n\
                                Remove the comma before `|:`.",
                            ),
                        );
                    }
                }

                TokenKind::BagEnd => {
                    let bag_end = self
                        .bump()
                        .expect("closing Bag delimiter must exist");

                    return Ok(Node::new(
                        NodeKind::Bag(BagLiteral { entries }),
                        Span {
                            start,
                            end: bag_end.pos + bag_end.lexeme.len(),
                        },
                    ));
                }

                TokenKind::Semicolon => {
                    return Err(
                        Diagnostic::error(
                            "invalid separator in Bag literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim Bag entries must be separated by commas, not semicolons.\n\
                            Example: `:| name: \"Rusty\", level: 42 |:`",
                        ),
                    );
                }

                TokenKind::Eof => {
                    return Err(
                        Diagnostic::error(
                            "unterminated Bag literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim expected a closing Bag delimiter `|:`.",
                        ),
                    );
                }

                _ => {
                    return Err(
                        Diagnostic::error(
                            "missing comma in Bag literal",
                            self.current_span(),
                        )
                        .with_help(
                            "Druim Bag entries must be separated by commas.\n\
                            Example: `:| name: \"Rusty\", level: 42 |:`",
                        ),
                    );
                }
            }
        }
    }

    fn parse_call_suffix(&mut self, callee: Node) -> Result<Node, Diagnostic> {
        let start = callee.span.start;

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

        let call_end = self
            .bump()
            .expect("closing function-call parenthesis must exist");

        Ok(Node::new(
            NodeKind::Call(Call {
                callee: Box::new(callee),
                args,
            }),
            Span {
                start,
                end: call_end.pos + call_end.lexeme.len(),
            },
        ))
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

fn is_math_expression(node: &Node) -> bool {
    matches!(
        &node.kind,
        NodeKind::Add(_, _)
            | NodeKind::Sub(_, _)
            | NodeKind::Mul(_, _)
            | NodeKind::Div(_, _)
            | NodeKind::Mod(_, _)
            | NodeKind::Neg(_)
    )
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
    Get,
    Has,
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
        TokenKind::Get => (22, 23, Get),
        TokenKind::Has => (22, 23, Has),

        _ => return None,
    })
}

fn build_infix(kind: Infix, lhs: Node, rhs: Node) -> Node {
    use Infix::*;

    let span = Span {
        start: lhs.span.start,
        end: rhs.span.end,
    };

    let kind = match kind {
        Add => NodeKind::Add(Box::new(lhs), Box::new(rhs)),
        Sub => NodeKind::Sub(Box::new(lhs), Box::new(rhs)),
        Mul => NodeKind::Mul(Box::new(lhs), Box::new(rhs)),
        Div => NodeKind::Div(Box::new(lhs), Box::new(rhs)),
        Mod => NodeKind::Mod(Box::new(lhs), Box::new(rhs)),

        Eq => NodeKind::Eq(Box::new(lhs), Box::new(rhs)),
        Ne => NodeKind::Ne(Box::new(lhs), Box::new(rhs)),
        Lt => NodeKind::Lt(Box::new(lhs), Box::new(rhs)),
        Le => NodeKind::Le(Box::new(lhs), Box::new(rhs)),
        Gt => NodeKind::Gt(Box::new(lhs), Box::new(rhs)),
        Ge => NodeKind::Ge(Box::new(lhs), Box::new(rhs)),

        And => NodeKind::And(Box::new(lhs), Box::new(rhs)),
        Or => NodeKind::Or(Box::new(lhs), Box::new(rhs)),

        Get => NodeKind::Get(Box::new(lhs), Box::new(rhs)),
        Has => NodeKind::Has(Box::new(lhs), Box::new(rhs)),
    };

    Node::new(kind, span)
}