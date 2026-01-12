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

            // Identifier or keyword
            if is_ident_start(ch) {
                let ident = self.read_while(is_ident_continue);
                let kind = match ident.as_str() {
                    "Num" => TokenKind::KwNum,
                    "Dec" => TokenKind::KwDec,
                    "Flag" => TokenKind::KwFlag,
                    "Text" => TokenKind::KwText,
                    "Emp" => TokenKind::KwEmp,
                    _ => TokenKind::Ident,
                };

                tokens.push(Token {
                    kind,
                    lexeme: ident,
                    pos: start,
                });
                continue;
            }

            // Number literal
            if ch.is_ascii_digit() {
                let number = self.read_number();
                let kind = if number.contains('.') {
                    TokenKind::DecLit
                } else {
                    TokenKind::NumLit
                };

                tokens.push(Token {
                    kind,
                    lexeme: number,
                    pos: start,
                });
                continue;
            }

            // Text literal
            if ch == '"' {
                let text = self.read_text(start)?;
                tokens.push(Token {
                    kind: TokenKind::TextLit,
                    lexeme: text,
                    pos: start,
                });
                continue;
            }

            // Block delimiters (must be before single ':')
            if self.match_str(":[") {
                tokens.push(tok(TokenKind::BlockExprStart, ":[", start));
                continue;
            }
            if self.match_str("]:") {
                tokens.push(tok(TokenKind::BlockExprEnd, "]:", start));
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
            if self.match_str("?=") {
                tokens.push(tok(TokenKind::QAssign, "?=", start));
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
            if self.match_str("&&") {
                tokens.push(tok(TokenKind::And, "&&", start));
                continue;
            }
            if self.match_str("||") {
                tokens.push(tok(TokenKind::Or, "||", start));
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


            // Colon-family operators (longest first)
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

            // Other operators / punctuation
            let kind = match ch {
                '=' => TokenKind::Define,
                '+' => TokenKind::Add,
                '-' => TokenKind::Sub,
                '*' => TokenKind::Mul,
                '/' => TokenKind::Div,
                '%' => TokenKind::Mod,
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                '!' => TokenKind::Not,
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

    fn read_number(&mut self) -> String {
        let start = self.pos;
        self.read_while(|c| c.is_ascii_digit());
        if !self.eof() && self.peek_char() == '.' {
            self.bump_char();
            self.read_while(|c| c.is_ascii_digit());
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

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
