use crate::compiler::token::TokenKind;

/// A half-open byte range into the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// High-level classification of parse errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// A token appeared where it is not valid.
    UnexpectedToken,

    /// A required token or construct was missing.
    ExpectedToken,

    /// A definition required an identifier.
    ExpectedIdentifier,

    /// The parser reached EOF unexpectedly.
    UnexpectedEof,

    /// A syntactically valid construct is illegal in this context.
    InvalidStatement,

    /// Tokens cannot form a valid expression in this context.
    InvalidExpression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}


/// Structured parse error with source location.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub expected: Option<&'static str>,
    pub found: Option<TokenKind>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub severity: Severity, // Note or Help (maybe Warning later)
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub help: Option<&'static str>,
    pub secondary: Vec<(Span, &'static str)>,
    pub notes: Vec<Note>,
}


#[derive(Debug, Clone)]
pub struct Source {
    text: String,
    line_starts: Vec<usize>,
}

impl Source {
    pub fn new(text: String) -> Self {
        let mut line_starts = vec![0];

        for (i, ch) in text.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }

        Self { text, line_starts }
    }

    pub fn line_col(&self, pos: usize) -> (usize, usize) {
        let line = match self.line_starts.binary_search(&pos) {
            Ok(i) => i,
            Err(i) => i - 1,
        };

        let col = pos - self.line_starts[line];
        (line + 1, col + 1)
    }

    pub fn line_text(&self, line: usize) -> &str {
        let start = self.line_starts[line - 1];
        let end = self
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(self.text.len());

        self.text[start..end]
            .trim_end_matches('\n')
    }

    pub fn is_newline_at(&self, pos: usize) -> bool {
        self.text
            .as_bytes()
            .get(pos)
            .map(|b| *b == b'\n')
            .unwrap_or(false)
    }
}


impl ParseError {
    pub fn unexpected_token(
        expected: &'static str,
        found: TokenKind,
        span: Span,
    ) -> Self {
        Self {
            kind: ErrorKind::UnexpectedToken,
            expected: Some(expected),
            found: Some(found),
            span,
        }
    }

    pub fn expected_identifier(span: Span) -> Self {
        Self {
            kind: ErrorKind::ExpectedIdentifier,
            expected: Some("identifier"),
            found: None,
            span,
        }
    }

    pub fn unexpected_eof(expected: &'static str, pos: usize) -> Self {
        Self {
            kind: ErrorKind::UnexpectedEof,
            expected: Some(expected),
            found: Some(TokenKind::Eof),
            span: Span { start: pos, end: pos },
        }
    }
}

impl From<ParseError> for Diagnostic {
    fn from(err: ParseError) -> Self {
        let message = match err.kind {
            ErrorKind::UnexpectedToken => "unexpected token",
            ErrorKind::ExpectedToken => "expected token",
            ErrorKind::ExpectedIdentifier => "expected identifier",
            ErrorKind::UnexpectedEof => "unexpected end of input",
            ErrorKind::InvalidStatement => "invalid statement",
            ErrorKind::InvalidExpression => "invalid expression",
        }.to_string();

        Diagnostic {
            severity: Severity::Error,
            message,
            span: err.span,
            help: err.expected,
            secondary: vec![],
            notes: vec![],            
        }
    }
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span,
            help: None,
            secondary: vec![],
            notes: vec![],
        }
    }

    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            span,
            help: None,
            secondary: vec![],
            notes: vec![],
        }
    }

    pub fn note(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Note,
            message: message.into(),
            span,
            help: None,
            secondary: vec![],
            notes: vec![],
        }
    }

    pub fn help(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Help,
            message: message.into(),
            span,
            help: None,
            secondary: vec![],
            notes: vec![],
        }
    }

    pub fn with_help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_secondary(mut self, span: Span, label: &'static str) -> Self {
        self.secondary.push((span, label));
        self
    }

    pub fn with_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }
}

impl Note {
    pub fn note(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: Severity::Note,
            message: message.into(),
            span,
        }
    }

    pub fn help(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: Severity::Help,
            message: message.into(),
            span,
        }
    }

    pub fn warning(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            span,
        }
    }

    pub fn error(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span,
        }
    }
}


