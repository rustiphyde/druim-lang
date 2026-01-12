use crate::compiler::token::{Token, TokenKind};

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar { ch: char, pos: usize },
    UnterminatedText { pos: usize },
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
                    "emp" => TokenKind::KwEmp,
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
                let text = self.read_text(start)?;
                tokens.push(Token {
                    kind: TokenKind::TextLit,
                    lexeme: text,
                    pos: start,
                });
                continue;
            }

            // ===== Multi-char operators (longest first) =====

            // ===== Block delimiters (must be before single ':') =====
            if self.match_str(":[") {
                tokens.push(tok(TokenKind::BlockExprStart, ":[", start));
                continue;
            }
            if self.match_str("]:") {
                tokens.push(tok(TokenKind::BlockExprEnd, "]:", start));
                continue;
            }
            if self.match_str("][") {
                tokens.push(tok(TokenKind::BlockExprChain, "][", start));
                continue;
            }

            if self.match_str(":{") {
                tokens.push(tok(TokenKind::BlockStmtStart, ":{", start));
                continue;
            }
            if self.match_str("}:") {
                tokens.push(tok(TokenKind::BlockStmtEnd, "}:", start));
                continue;
            }
            if self.match_str("}{") {
                tokens.push(tok(TokenKind::BlockStmtChain, "}{", start));
                continue;
            }

            if self.match_str(":(") {
                tokens.push(tok(TokenKind::BlockFuncStart, ":(", start));
                continue;
            }
            if self.match_str("):") {
                tokens.push(tok(TokenKind::BlockFuncEnd, "):", start));
                continue;
            }
            if self.match_str(")(") {
                tokens.push(tok(TokenKind::BlockFuncChain, ")(", start));
                continue;
            }

            if self.match_str(":<") {
                tokens.push(tok(TokenKind::BlockArrayStart, ":<", start));
                continue;
            }
            if self.match_str(">:") {
                tokens.push(tok(TokenKind::BlockArrayEnd, ">:", start));
                continue;
            }
            if self.match_str("><") {
                tokens.push(tok(TokenKind::BlockArrayChain, "><", start));
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
                tokens.push(tok(TokenKind::Pipe, "|>", start));
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
            if self.match_str("<=") {
                tokens.push(tok(TokenKind::Le, "<=", start));
                continue;
            }
            if self.match_str(">=") {
                tokens.push(tok(TokenKind::Ge, ">=", start));
                continue;
            }

            if self.match_str("&?") {
                tokens.push(tok(TokenKind::And, "&?", start));
                continue;
            }
            if self.match_str("|?") {
                tokens.push(tok(TokenKind::Or, "|?", start));
                continue;
            }
            if self.match_str("!?") {
                tokens.push(tok(TokenKind::Not, "!?", start));
                continue;
            }

            if self.match_str("->") {
                tokens.push(tok(TokenKind::ArrowR, "->", start));
                continue;
            }
            if self.match_str("<-") {
                tokens.push(tok(TokenKind::ArrowL, "<-", start));
                continue;
            }

            // ===== Colon-family operators (longest first) =====
            if self.match_str("::") {
                tokens.push(tok(TokenKind::Scope, "::", start));
                continue;
            }
            if self.match_str(":=") {
                tokens.push(tok(TokenKind::Bind, ":=", start));
                continue;
            }
            if self.match_str(":?") {
                tokens.push(tok(TokenKind::Present, ":?", start));
                continue;
            }
            if self.match_str(":>") {
                tokens.push(tok(TokenKind::Cast, ":>", start));
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
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                _ => {
                    return Err(LexError::UnexpectedChar {
                        ch,
                        pos: self.pos,
                    })
                }
            };

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


    fn read_text(&mut self, start_pos: usize) -> Result<String, LexError> {
        // consume opening quote
        self.bump_char();
        let start = self.pos;

        while !self.eof() && self.peek_char() != '"' {
            self.bump_char();
        }

        if self.eof() {
            return Err(LexError::UnterminatedText { pos: start_pos });
        }

        let text = self.src[start..self.pos].to_string();
        self.bump_char(); // closing quote
        Ok(text)
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

fn tok(kind: TokenKind, lex: &str, pos: usize) -> Token {
    Token {
        kind,
        lexeme: lex.to_string(),
        pos,
    }
}
