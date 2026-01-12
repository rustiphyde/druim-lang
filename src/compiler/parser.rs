use crate::compiler::ast::{Expr, Literal, Program, Stmt};
use crate::compiler::error::{Span, Diagnostic};
use crate::compiler::token::{Token, TokenKind};
use std::collections::HashSet;


pub struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
    scopes: Vec<HashSet<String>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            index: 0,
            scopes: vec![HashSet::new()],
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut stmts = Vec::new();

        while self.peek_kind() != TokenKind::Eof {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        Ok(Program { stmts })
    }

    pub fn parse_stmt(&mut self) -> Result<Stmt, Diagnostic> {

        if self.peek_kind() == TokenKind::BlockStmtStart {
            return self.parse_block_stmt();
        }
        // DefineEmpty must start with identifier
        if self.peek_kind() == TokenKind::DefineEmpty {
            return Err(
                Diagnostic::error("invalid define statement", self.current_span())
                    .with_help("define statements must start with an identifier")
            );
        }

        // Special-case Define first
        if self.peek_kind() == TokenKind::Ident {
            if let Some(next) = self.tokens.get(self.index + 1) {

                // name =;
                if let Some(next) = self.tokens.get(self.index + 1) {
                    if next.kind == TokenKind::DefineEmpty {
                        let name_tok = self.bump().unwrap(); // ident
                        let name = name_tok.lexeme.clone();
                        self.bump(); // consume =;

                        // Disallow chaining: a =; = b;
                        if self.peek_kind() == TokenKind::Define || self.peek_kind() == TokenKind::DefineEmpty {
                            return Err(
                                Diagnostic::error("invalid define statement", self.current_span())
                                    .with_help("define statements cannot be chained")
                            );
                        }

                        return Ok(Stmt::DefineEmpty { name });
                    }
                }

                // name = expr;
                if next.kind == TokenKind::Define {
                    let name_tok = self.bump().unwrap(); // ident
                    let name = name_tok.lexeme.clone();

                    self.bump(); // consume '='

                    let value = self.parse_expr()?;

                    // Disallow chained define: a = b = c;
                    if self.peek_kind() == TokenKind::Define {
                        return Err(
                            Diagnostic::error("invalid define statement", self.current_span())
                                .with_help("define statements cannot be chained")
                        );
                    }

                    self.expect(TokenKind::Semicolon, "`;`")?;

                    return Ok(Stmt::Define { name, value });

                }
            }
        }

        // Otherwise: expression-based flow statement
        let lhs = self.parse_expr()?;
        
        let stmt = match self.peek_kind() {
        TokenKind::ArrowL => {
            self.bump();
            let rhs = self.parse_expr()?;
            Stmt::AssignFrom { target: lhs, source: rhs }
        }

        TokenKind::ArrowR => {
            self.bump();
            let rhs = self.parse_expr()?;
            Stmt::SendTo { value: lhs, destination: rhs }
        }

        // `=` is not a general statement delimiter. It is ONLY valid as a Define or DefineEmpty,
        // and Define and DefineEmpty statements must begin with an identifier.
        TokenKind::Define | TokenKind::DefineEmpty => {
            return Err(
                Diagnostic::error("invalid define statement", self.current_span())
                    .with_help("define statements must start with an identifier")
            );
        }

    _ => {
        return Err(
            Diagnostic::error("unexpected token", self.current_span())
                .with_help("expected `<-` or `->`")
        );
    }
};


        self.expect(TokenKind::Semicolon, "`;`")?;
        Ok(stmt)
    }

    fn parse_block_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        // consume :{
        let start_span = self.current_span();
        self.bump(); // BlockStmtStart

        self.scopes.push(HashSet::new());

        let mut stmts = Vec::new();

        // parse statements until we hit }:
        while self.peek_kind() != TokenKind::BlockStmtEnd {
            if self.peek_kind() == TokenKind::Eof {
                return Err(Diagnostic::error(
                    "unexpected end of block",
                    start_span,
                ).with_help("expected `}:` to close the block"));
            }


            let stmt = self.parse_stmt()?;
            stmts.push(stmt);

        }

        // consume }:
        self.bump();

        self.scopes.pop();

        Ok(Stmt::Block { stmts })
    }




    pub fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        self.parse_bp(0)
    }

    // ===== Pratt parser =====

    fn parse_bp(&mut self, min_bp: u8) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_prefix()?;

        loop {
            let op = self.peek_kind();

            let Some((l_bp, r_bp, infix_kind)) = infix_binding_power(op) else {
                break;
            };

            if l_bp < min_bp {
                break;
            }

            // consume operator token
            self.bump();

            // function call: lhs(args...)
            if infix_kind == Infix::Call {
                let args = self.parse_args()?;
                lhs = Expr::Call {
                    callee: Box::new(lhs),
                    args,
                };
                continue;
            }

            let rhs = self.parse_bp(r_bp)?;
            lhs = build_infix(infix_kind, lhs, rhs);
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, Diagnostic> {
        let span_start = self.current_span().start;
        let tok = self.bump().ok_or_else(|| {
            Diagnostic::error("unexpected end of input", Span {
                start: span_start,
                end: span_start,
            }).with_help("expected expression")
        })?;


        match tok.kind {

            TokenKind::BlockExprStart => {
                let inner = self.parse_expr()?;

                // HARD RULE:
                // Expression blocks must close immediately after the expression.
                // No statements, no defines, no semicolons.
                if self.peek_kind() != TokenKind::BlockExprEnd {
                    return Err(
                        Diagnostic::error("unexpected token", self.current_span())
                            .with_help("expected expression")
                    );
                }

                self.bump(); // consume BlockExprEnd

                Ok(Expr::BlockExpr {
                    expr: Box::new(inner),
                })
            }



            TokenKind::Ident => Ok(Expr::Ident(tok.lexeme.clone())),

            TokenKind::NumLit => {
                let n = tok.lexeme.parse::<i64>().unwrap_or(0);
                Ok(Expr::Lit(Literal::Num(n)))
            }

            // IMPORTANT: Dec literal stays as text for now. We can parse/validate later,
            // but keeping the original lexeme preserves fidelity and avoids float pitfalls.
            TokenKind::DecLit => Ok(Expr::Lit(Literal::Dec(tok.lexeme.clone()))),

            TokenKind::TextLit => Ok(Expr::Lit(Literal::Text(tok.lexeme.clone()))),

            // Emp literal: keyword Emp
            TokenKind::KwEmp => Ok(Expr::Lit(Literal::Emp)),

            // unary operators
            TokenKind::Not => {
                let rhs = self.parse_bp(PREFIX_BP)?;
                Ok(Expr::Not(Box::new(rhs)))
            }

            TokenKind::Sub => {
                let rhs = self.parse_bp(PREFIX_BP)?;
                Ok(Expr::Neg(Box::new(rhs)))
            }

            // grouping
            TokenKind::LParen => {
                let e = self.parse_bp(0)?;
                self.expect(TokenKind::RParen, "`)`")?;
                Ok(e)
            }

            _ => Err(
                Diagnostic::error("invalid expression", Span {
                    start: tok.pos,
                    end: tok.pos + tok.lexeme.len(),
                }).with_help("expected expression")
            ),


        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, Diagnostic> {
        let mut args = Vec::new();

        // immediate close: f()
        if self.peek_kind() == TokenKind::RParen {
            self.bump();
            return Ok(args);
        }

        loop {
            args.push(self.parse_bp(0)?);

            match self.peek_kind() {
                TokenKind::Comma => {
                    self.bump();
                }
                TokenKind::RParen => {
                    self.bump();
                    break;
                }
                _k => {
                    return Err(
                        Diagnostic::error("unexpected token", self.current_span())
                            .with_help("expected `,` or `)`")
                    );

                }
            }
        }

        Ok(args)
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

    fn define_name(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn is_defined(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.contains(name))
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

    // Assignment / binding (right associative) — expression forms (not stmt forms)
    QAssign,
    Bind,

    // Colon semantics
    Scope,
    Present,
    Cast,

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
        TokenKind::Scope => (22, 23, Scope),
        TokenKind::Present => (22, 23, Present),
        TokenKind::Cast => (22, 23, Cast),

        // pipe
        TokenKind::Pipe => (20, 21, Pipe),

        // assignment-like (expression forms): right associative
        TokenKind::QAssign => (10, 9, QAssign),
        TokenKind::Bind => (10, 9, Bind),

        _ => return None,
    })
}

fn build_infix(kind: Infix, lhs: Expr, rhs: Expr) -> Expr {
    use Infix::*;

    match kind {
        Add => Expr::Add(Box::new(lhs), Box::new(rhs)),
        Sub => Expr::Sub(Box::new(lhs), Box::new(rhs)),
        Mul => Expr::Mul(Box::new(lhs), Box::new(rhs)),
        Div => Expr::Div(Box::new(lhs), Box::new(rhs)),
        Mod => Expr::Mod(Box::new(lhs), Box::new(rhs)),

        Eq => Expr::Eq(Box::new(lhs), Box::new(rhs)),
        Ne => Expr::Ne(Box::new(lhs), Box::new(rhs)),
        Lt => Expr::Lt(Box::new(lhs), Box::new(rhs)),
        Le => Expr::Le(Box::new(lhs), Box::new(rhs)),
        Gt => Expr::Gt(Box::new(lhs), Box::new(rhs)),
        Ge => Expr::Ge(Box::new(lhs), Box::new(rhs)),

        And => Expr::And(Box::new(lhs), Box::new(rhs)),
        Or => Expr::Or(Box::new(lhs), Box::new(rhs)),

        QAssign => Expr::QAssign(Box::new(lhs), Box::new(rhs)),
        Bind => Expr::Bind(Box::new(lhs), Box::new(rhs)),

        Scope => Expr::Scope(Box::new(lhs), Box::new(rhs)),
        Present => Expr::Present(Box::new(lhs), Box::new(rhs)),
        Cast => Expr::Cast(Box::new(lhs), Box::new(rhs)),

        Pipe => Expr::Pipe(Box::new(lhs), Box::new(rhs)),

        Call => unreachable!("Call is handled in parse_bp"),
    }
}
