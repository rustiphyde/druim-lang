use crate::compiler::token::{Token, TokenKind};

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar { ch: char, pos: usize },
    UnterminatedText { pos: usize },
    UnterminatedInterpolation { pos: usize },
    UnterminatedSingleComment { pos: usize },
    UnterminatedMultiComment { pos: usize },
    CommentInFunctionSyntax { pos: usize },
}

pub struct Lexer<'a> {
    src: &'a str,
    pos: usize, // byte offset
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        let mut in_function_parameters = false;
        let mut call_paren_depth = 0usize;

        while !self.eof() {
            self.skip_whitespace();

            if self.eof() {
                break;
            }

            let start = self.pos;
            let ch = self.peek_char();

            // ===== Digit-starting: NumLit, DecLit, or digit-leading Ident =====
            //
            // Rules:
            // - If it starts with digits and continues with letters/_ -> Ident (e.g., 1a, 9lives, 123_456)
            // - If it's only digits -> NumLit
            // - Decimals are strictly digits '.' digits (e.g., 3.14)
            // - Invalid decimals error: "1.", "1..2"
            if ch.is_ascii_digit() {
                let start = self.pos;

                // First consume the leading digit run.
                self.read_while(|c| c.is_ascii_digit());

                // Decimal form: digits '.' digits
                if !self.eof() && self.peek_char() == '.' {
                    self.bump_char(); // consume '.'

                    // Require at least one digit after the decimal point.
                    if self.eof() || !self.peek_char().is_ascii_digit() {
                        return Err(LexError::UnexpectedChar {
                            ch: '.',
                            pos: self.pos - 1, // position of '.'
                        });
                    }

                    self.read_while(|c| c.is_ascii_digit());

                    tokens.push(Token {
                        kind: TokenKind::DecLit,
                        lexeme: self.src[start..self.pos].to_string(),
                        pos: start,
                    });

                    continue;
                }

                // If the next char is identifier-continue, this is a digit-leading identifier.
                if !self.eof() {
                    let next = self.peek_char();
                    if next.is_ascii_alphabetic() || next == '_' {
                        self.read_while(|c| c.is_ascii_alphanumeric() || c == '_');

                        tokens.push(Token {
                            kind: TokenKind::Ident,
                            lexeme: self.src[start..self.pos].to_string(),
                            pos: start,
                        });

                        continue;
                    }
                }

                // Otherwise it is pure digits.
                tokens.push(Token {
                    kind: TokenKind::NumLit,
                    lexeme: self.src[start..self.pos].to_string(),
                    pos: start,
                });

                continue;
            }

            // ===== Identifier or keyword (non-digit start) =====
            if ch.is_ascii_alphabetic() || ch == '_' {
                let text = self.read_while(|c| c.is_ascii_alphanumeric() || c == '_');

                let kind = match text.as_str() {
                    "num" => TokenKind::KwNum,
                    "dec" => TokenKind::KwDec,
                    "flag" => TokenKind::KwFlag,
                    "text" => TokenKind::KwText,
                    "void" => TokenKind::KwVoid,
                    "fn" => TokenKind::KwFn,
                    "ret" => TokenKind::KwRet,
                    "loc" => TokenKind::KwLoc,
                    "glo" => TokenKind::KwGlo,
                    "stone" => TokenKind::KwStone,
                    "true" | "false" => TokenKind::FlagLit,
                    _ => TokenKind::Ident,
                };

                tokens.push(Token {
                    kind,
                    lexeme: text,
                    pos: start,
                });

                continue;
            }

            // ===== Text literal =====
            if ch == '"' {
                tokens.extend(self.read_text_tokens(start)?);
                continue;
            }
            // ===== Multi-char operators (longest first) =====

            // ===== Program boundary =====
            if self.match_str(":-:-:") {
                tokens.push(tok(
                    TokenKind::ProgramBoundary,
                    ":-:-:",
                    start,
                ));
                continue;
            }

            // ===== Comments =====
            //
            // Comments are consumed by the lexer and never emitted as tokens.
            //
            // Comments are forbidden inside:
            // - function parameter lists `:( ... )(`
            // - function call argument lists `( ... )`
            //
            // Longest form must be checked first because `:--` begins with `:-`.

            if self.src[self.pos..].starts_with(":--") {
                if in_function_parameters || call_paren_depth > 0 {
                    return Err(
                        LexError::CommentInFunctionSyntax {
                            pos: start,
                        },
                    );
                }

                self.pos += 3;
                self.skip_comment("--:", start, true)?;
                continue;
            }

            if self.src[self.pos..].starts_with(":-") {
                if in_function_parameters || call_paren_depth > 0 {
                    return Err(
                        LexError::CommentInFunctionSyntax {
                            pos: start,
                        },
                    );
                }

                self.pos += 2;
                self.skip_comment("-:", start, false)?;
                continue;
            }

            // ===== Block delimiters (must be before single ':') =====
            if self.match_str(":[") {
                tokens.push(tok(TokenKind::BoxStart, ":[", start));
                continue;
            }
            if self.src[self.pos..].starts_with("]::")
                || self.src[self.pos..].starts_with("]:?")
            {
                let start = self.pos;

                self.pos += 1;

                tokens.push(Token {
                    kind: TokenKind::RBracket,
                    lexeme: "]".to_string(),
                    pos: start,
                });

                continue;
            }
            if self.match_str("]:") {
                tokens.push(tok(TokenKind::BoxEnd, "]:", start));
                continue;
            }
            if self.match_str(":|") {
                tokens.push(tok(TokenKind::BagStart, ":|", start));
                continue;
            }
            if self.match_str("|:") {
                tokens.push(tok(TokenKind::BagEnd, "|:", start));
                continue;
            }
            if self.match_str(":{") {
                tokens.push(tok(TokenKind::BlockStart, ":{", start));
                continue;
            }
            if self.match_str("}:") {
                tokens.push(tok(TokenKind::BlockEnd, "}:", start));
                continue;
            }
            if self.match_str("}{") {
                tokens.push(tok(TokenKind::BlockChain, "}{", start));
                continue;
            }
            if self.match_str(":<") {
                tokens.push(tok(TokenKind::LoopStart, ":<", start));
                continue;
            }
            if self.match_str(">?<") {
                tokens.push(tok(TokenKind::LoopSplit, ">?<", start));
                continue;
            }
            if self.match_str(">:") {
                tokens.push(tok(TokenKind::LoopEnd, ">:", start));
                continue;
            }

            if self.match_str(":(") {
                in_function_parameters = true;

                tokens.push(tok(
                    TokenKind::FuncStart,
                    ":(",
                    start,
                ));

                continue;
            }

            if self.match_str("):") {
                tokens.push(tok(
                    TokenKind::FuncEnd,
                    "):",
                    start,
                ));

                continue;
            }

            if self.match_str(")(") {
                in_function_parameters = false;

                tokens.push(tok(
                    TokenKind::FuncChain,
                    ")(",
                    start,
                ));

                continue;
            }


            // ===== Other multi-char operators =====
            if self.match_str("?=") {
                tokens.push(tok(TokenKind::Guard, "?=", start));
                continue;
            }
            if self.match_str("=;") {
                tokens.push(tok(TokenKind::DefineEmpty, "=;", start));
                continue;
            }
            if self.match_str("|>") {
                tokens.push(tok(TokenKind::Print, "|>", start));
                continue;
            }

            if self.match_str("==") {
                tokens.push(tok(TokenKind::Eq, "==", start));
                continue;
            }
            if self.match_str("!=") {
                tokens.push(tok(TokenKind::Ne, "!=", start));
                continue;
            }
            if self.match_str("<<") {
                tokens.push(tok(TokenKind::Mutate, "<<", start));
                continue;
            }
            if self.match_str("<=") {
                tokens.push(tok(TokenKind::Le, "<=", start));
                continue;
            }
            if self.match_str(">=") {
                tokens.push(tok(TokenKind::Ge, ">=", start));
                continue;
            }

            if self.match_str("&&") {
                tokens.push(tok(TokenKind::And, "&&", start));
                continue;
            }
            if self.match_str("||") {
                tokens.push(tok(TokenKind::Or, "||", start));
                continue;
            }

            // ===== Colon-family operators (longest first) =====
            if self.match_str("::") {
                tokens.push(tok(TokenKind::Get, "::", start));
                continue;
            }
            if self.match_str(":?") {
                tokens.push(tok(TokenKind::Has, ":?", start));
                continue;
            }
            if self.match_str(":=") {
                tokens.push(tok(TokenKind::Copy, ":=", start));
                continue;
            }
            if self.match_str(":>") {
                tokens.push(tok(TokenKind::Bind, ":>", start));
                continue;
            }
            if self.match_char(':') {
                tokens.push(tok(TokenKind::Colon, ":", start));
                continue;
            }

            // ===== Single-char operators / punctuation =====
            let kind = match ch {
                '=' => TokenKind::Define,
                '+' => TokenKind::Add,
                '-' => TokenKind::Sub,
                '*' => TokenKind::Mul,
                '/' => TokenKind::Div,
                '%' => TokenKind::Mod,
                '>' => TokenKind::Gt,
                '<' => TokenKind::Lt,
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '[' => TokenKind::LBracket,
                ']' => TokenKind::RBracket,
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                '!' => TokenKind::Not,
                _ => {
                    return Err(LexError::UnexpectedChar {
                        ch,
                        pos: self.pos,
                    });
                }
            };

            if kind == TokenKind::LParen {
                let starts_call =
                    call_paren_depth > 0
                        || tokens
                            .last()
                            .is_some_and(
                                |token| token.kind == TokenKind::Ident
                            );

                if starts_call {
                    call_paren_depth += 1;
                }
            } else if kind == TokenKind::RParen
                && call_paren_depth > 0
            {
                call_paren_depth -= 1;
            }

            self.bump_char();
            tokens.push(Token {
                kind,
                lexeme: ch.to_string(),
                pos: start,
            });
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            pos: self.pos,
        });

        Ok(tokens)
    }

    // ===== helpers =====

    fn skip_comment(
    &mut self,
    closing: &str,
    start_pos: usize,
    multiline: bool,
) -> Result<(), LexError> {
    while !self.eof() {
        if self.src[self.pos..].starts_with(closing) {
            self.pos += closing.len();
            return Ok(());
        }

        if !multiline && self.peek_char() == '\n' {
            return Err(LexError::UnterminatedSingleComment {
                pos: start_pos,
            });
        }

        self.bump_char();
    }

    if multiline {
        Err(LexError::UnterminatedMultiComment {
            pos: start_pos,
        })
    } else {
        Err(LexError::UnterminatedSingleComment {
            pos: start_pos,
        })
    }
}

    fn skip_whitespace(&mut self) {
        while !self.eof() && self.peek_char().is_whitespace() {
            self.bump_char();
        }
    }

    fn read_while<F>(&mut self, cond: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let start = self.pos;
        while !self.eof() && cond(self.peek_char()) {
            self.bump_char();
        }
        self.src[start..self.pos].to_string()
    }


    fn read_text_tokens(
        &mut self,
        start_pos: usize,
    ) -> Result<Vec<Token>, LexError> {
        self.bump_char(); // opening `"`

        let content_start = self.pos;
        let mut segment_start = self.pos;
        let mut tokens = Vec::new();
        let mut interpolated = false;

        while !self.eof() {
            // Interpolation only has special meaning while we are inside text.
            if self.src[self.pos..].starts_with(":.") {
                if !interpolated {
                    interpolated = true;

                    tokens.push(Token {
                        kind: TokenKind::TextStart,
                        lexeme: "\"".to_string(),
                        pos: start_pos,
                    });
                }

                if segment_start < self.pos {
                    tokens.push(Token {
                        kind: TokenKind::TextLit,
                        lexeme: self.src[segment_start..self.pos].to_string(),
                        pos: segment_start,
                    });
                }

                let interpolation_start = self.pos;

                tokens.push(Token {
                    kind: TokenKind::InterpStart,
                    lexeme: ":.".to_string(),
                    pos: interpolation_start,
                });

                self.pos += 2; // consume `:.`

                let expression_start = self.pos;

                let expression_end = self
                    .find_interpolation_end(expression_start)
                    .ok_or(
                        LexError::UnterminatedInterpolation {
                            pos: interpolation_start,
                        },
                    )?;

                let expression_source =
                    &self.src[expression_start..expression_end];

                let mut expression_lexer =
                    Lexer::new(expression_source);

                let mut expression_tokens =
                    expression_lexer
                        .tokenize()
                        .map_err(|error| {
                            offset_lex_error(
                                error,
                                expression_start,
                            )
                        })?;

                // The nested lexer always adds Eof. It belongs to the
                // interpolation substring, not the surrounding program.
                if expression_tokens
                    .last()
                    .is_some_and(|token| token.kind == TokenKind::Eof)
                {
                    expression_tokens.pop();
                }

                for token in &mut expression_tokens {
                    token.pos += expression_start;
                }

                tokens.extend(expression_tokens);

                self.pos = expression_end;

                tokens.push(Token {
                    kind: TokenKind::InterpEnd,
                    lexeme: ".:".to_string(),
                    pos: self.pos,
                });

                self.pos += 2; // consume `.:`

                segment_start = self.pos;

                continue;
            }

            if self.peek_char() == '"' {
                let closing_quote = self.pos;
                self.bump_char();

                // No interpolation occurred. Preserve the original
                // one-token representation for ordinary text.
                if !interpolated {
                    return Ok(vec![Token {
                        kind: TokenKind::TextLit,
                        lexeme: self.src[
                            content_start..closing_quote
                        ]
                        .to_string(),
                        pos: start_pos,
                    }]);
                }

                if segment_start < closing_quote {
                    tokens.push(Token {
                        kind: TokenKind::TextLit,
                        lexeme: self.src[
                            segment_start..closing_quote
                        ]
                        .to_string(),
                        pos: segment_start,
                    });
                }

                tokens.push(Token {
                    kind: TokenKind::TextEnd,
                    lexeme: "\"".to_string(),
                    pos: closing_quote,
                });

                return Ok(tokens);
            }

            self.bump_char();
        }

        Err(LexError::UnterminatedText {
            pos: start_pos,
        })
    }

    fn find_interpolation_end(
        &self,
        start_pos: usize,
    ) -> Option<usize> {
        let mut pos = start_pos;
        let mut in_text = false;

        while pos < self.src.len() {
            let rest = &self.src[pos..];
            let ch = rest.chars().next().unwrap();

            if ch == '"' {
                in_text = !in_text;
                pos += ch.len_utf8();
                continue;
            }

            if !in_text && rest.starts_with(".:") {
                return Some(pos);
            }

            pos += ch.len_utf8();
        }

        None
    }

    fn match_str(&mut self, s: &str) -> bool {
        if self.src[self.pos..].starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn match_char(&mut self, c: char) -> bool {
        if !self.eof() && self.peek_char() == c {
            self.bump_char();
            true
        } else {
            false
        }
    }

    fn bump_char(&mut self) {
        let c = self.peek_char();
        self.pos += c.len_utf8();
    }

    fn peek_char(&self) -> char {
        self.src[self.pos..].chars().next().unwrap()
    }

    fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }
}

fn offset_lex_error(
    error: LexError,
    offset: usize,
) -> LexError {
    match error {
        LexError::UnexpectedChar { ch, pos } => {
            LexError::UnexpectedChar {
                ch,
                pos: pos + offset,
            }
        }

        LexError::UnterminatedText { pos } => {
            LexError::UnterminatedText {
                pos: pos + offset,
            }
        }

        LexError::UnterminatedInterpolation { pos } => {
            LexError::UnterminatedInterpolation {
                pos: pos + offset,
            }
        }

        LexError::UnterminatedSingleComment { pos } => {
            LexError::UnterminatedSingleComment {
                pos: pos + offset,
            }
        }

        LexError::UnterminatedMultiComment { pos } => {
            LexError::UnterminatedMultiComment {
                pos: pos + offset,
            }
        }

        LexError::CommentInFunctionSyntax { pos } => {
            LexError::CommentInFunctionSyntax {
                pos: pos + offset,
            }
        }
    }
}

fn tok(kind: TokenKind, lex: &str, pos: usize) -> Token {
    Token {
        kind,
        lexeme: lex.to_string(),
        pos,
    }
}
