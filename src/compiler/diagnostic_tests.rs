use crate::compiler::diagnostic::render;
use crate::compiler::error::{Diagnostic, Severity, Source, Span, Note};

fn assert_render(diag: &Diagnostic, source: &Source, expected: &str) {
    let got = render(diag, source);
    assert_eq!(
        got,
        expected,
        "\n--- expected ---\n{}\n--- got ---\n{}\n",
        expected,
        got
    );
}

#[test]
fn render_simple_error_single_caret() {
    let source = Source::new("let x = ;\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unexpected token".to_string(),
        span: Span { start: 8, end: 9 },
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: unexpected token
 --> line 1, column 9
  |
1 | let x = ;
  |         ^
",
    );
}


#[test]
fn render_error_with_help() {
    let source = Source::new("define x =\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "expected expression".to_string(),
        span: Span { start: 10, end: 10 },
        help: Some("expressions cannot be empty"),
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: expected expression
 --> line 1, column 11
  |
1 | define x =
  |           ^

help: expressions cannot be empty
",
    );
}

#[test]
fn render_multi_character_span() {
    // Source: "let total = 123;\n"
    // Indexes: l0 e1 t2 ' '3 t4 o5 t6 a7 l8 ' '9 =10 ' '11 1(12) 2(13) 3(14) ;15 \n16
    let source = Source::new("let total = 123;\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "invalid number".to_string(),
        span: Span { start: 12, end: 15 }, // highlights "123"
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: invalid number
 --> line 1, column 13
  |
1 | let total = 123;
  |             ^^^
",
    );
}

#[test]
fn render_multi_digit_line_number() {
    // 11 lines total; error is on line 10
    let source_text = "\
line 1
line 2
line 3
line 4
line 5
line 6
line 7
line 8
line 9
bad stuff
line 11
";
    let source = Source::new(source_text.to_string());

    // "bad stuff" starts at the beginning of line 10
    // Compute the byte index manually:
    // Each "line X\n" is 7 bytes for lines 1–9 ("line 1\n" .. "line 9\n")
    // 9 * 7 = 63, so line 10 starts at byte 63
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "invalid syntax".to_string(),
        span: Span { start: 63, end: 66 }, // highlights "bad"
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: invalid syntax
 --> line 10, column 1
   |
10 | bad stuff
   | ^^^
",
    );
}

#[test]
fn render_warning_severity() {
    let source = Source::new("let x = 1;\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Warning,
        message: "unused variable".to_string(),
        span: Span { start: 4, end: 5 }, // highlights "x"
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
warning: unused variable
 --> line 1, column 5
  |
1 | let x = 1;
  |     ^
",
    );
}

#[test]
fn render_span_at_column_one() {
    let source = Source::new("oops = 1;\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unexpected identifier".to_string(),
        span: Span { start: 0, end: 4 }, // highlights "oops"
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: unexpected identifier
 --> line 1, column 1
  |
1 | oops = 1;
  | ^^^^
",
    );
}

#[test]
fn render_span_at_end_of_line_clamped() {
    let source = Source::new("value = 42\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unexpected end of input".to_string(),
        // Start at the last character ('2'), end goes past the line
        span: Span { start: 9, end: 20 }, // start on '2', not '\n'
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: unexpected end of input
 --> line 1, column 10
  |
1 | value = 42
  |          ^
",
    );
}

#[test]
fn render_note_without_source_span() {
    let source = Source::new("let x = 1;\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Note,
        message: "this value is inferred".to_string(),
        span: Span { start: 0, end: 0 }, // ignored for note-only diagnostics
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    let got = render(&diag, &source);

    assert_eq!(
        got,
        "\
note: this value is inferred
"
    );
}

#[test]
fn render_error_in_multi_line_source() {
    let source = Source::new(
        "\
let a = 1;
let b = ;
let c = 3;
"
        .to_string(),
    );

    // Error is on line 2: "let b = ;"
    // Indexes:
    // line 1: "let a = 1;\n" -> 11 bytes
    // line 2 starts at byte 11
    // "let b = ;" -> ';' is at byte 18
    let diag = Diagnostic {
        severity: Severity::Error,
        message: "expected expression".to_string(),
        span: Span { start: 19, end: 20 },
        help: Some("expressions cannot be empty"),
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: expected expression
 --> line 2, column 9
  |
2 | let b = ;
  |         ^

help: expressions cannot be empty
",
    );
}

#[test]
fn render_error_with_secondary_span_label() {
    let source = Source::new(
        "\
let total = price * qty;
let price = 10;
"
        .to_string(),
    );

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable `qty`".to_string(),
        span: Span { start: 20, end: 23 }, // "qty"
        help: None,
        secondary: vec![(
            Span { start: 11 , end: 19 },
            "defined here",
        )],
        notes: vec![],

    };

    assert_render(
        &diag,
        &source,
        "\
error: unknown variable `qty`
 --> line 1, column 21
  |
1 | let total = price * qty;
  |                     ^^^
  |             -------- defined here
",
    );
}

#[test]
fn render_error_with_multiple_secondary_labels() {
    let source = Source::new(
        "\
let total = price * qty + tax;
let price = 10;
let tax = 2;
"
        .to_string(),
    );

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variables".to_string(),
        span: Span { start: 20, end: 29 }, // "qty + tax"
        help: None,
        secondary: vec![
            (Span { start: 12, end: 17 }, "defined here"), // price
            (Span { start: 33, end: 36 }, "defined here"), // tax
        ],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: unknown variables
 --> line 1, column 21
  |
1 | let total = price * qty + tax;
  |                     ^^^^^^^^^
  |             -------- defined here
  |             -------- defined here
",
    );
}

#[test]
fn render_error_with_note_and_help() {
    let source = Source::new("x = y;\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable `y`".to_string(),
        span: Span { start: 4, end: 5 },
        help: None,
        secondary: vec![],
        notes: vec![
            Note {
                severity: Severity::Note,
                message: "`y` must be declared before use".to_string(),
                span: None,
            },
            Note {
                severity: Severity::Help,
                message: "try defining `y` earlier in the file".to_string(),
                span: None,
            },
        ],
    };

    assert_render(
        &diag,
        &source,
        "\
error: unknown variable `y`
 --> line 1, column 5
  |
1 | x = y;
  |     ^

note: `y` must be declared before use

help: try defining `y` earlier in the file
",
    );
}

#[test]
fn render_embedded_note_with_source_span() {
    let source = Source::new(
        "\
let total = price * qty;
let price = 10;
"
        .to_string(),
    );

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable `qty`".to_string(),
        span: Span { start: 20, end: 23 },
        help: None,
        secondary: vec![],
        notes: vec![
            Note {
                severity: Severity::Note,
                message: "`price` is defined here".to_string(),
                span: Some(Span { start: 12, end: 17 }),
            }
        ],
    };

    assert_render(
        &diag,
        &source,
        "\
error: unknown variable `qty`
 --> line 1, column 21
  |
1 | let total = price * qty;
  |                     ^^^

note: `price` is defined here
 --> line 1, column 13
  |
1 | let total = price * qty;
  |             ^^^^^
",
    );
}

#[test]
fn render_error_with_multiple_notes_mixed_spans() {
    let source = Source::new(
        "\
let total = price * qty;
let price = 10;
"
        .to_string(),
    );

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable `qty`".to_string(),
        span: Span { start: 20, end: 23 }, // qty
        help: Some("declare `qty` before use"),
        secondary: vec![],
        notes: vec![
            Note {
                severity: Severity::Note,
                message: "`price` is defined here".to_string(),
                span: Some(Span { start: 12, end: 17 }), // price
            },
            Note {
                severity: Severity::Note,
                message: "`qty` was never declared".to_string(),
                span: None,
            },
        ],
    };

    assert_render(
        &diag,
        &source,
        "\
error: unknown variable `qty`
 --> line 1, column 21
  |
1 | let total = price * qty;
  |                     ^^^

note: `price` is defined here
 --> line 1, column 13
  |
1 | let total = price * qty;
  |             ^^^^^

note: `qty` was never declared

help: declare `qty` before use
",
    );
}

#[test]
fn caret_renders_for_zero_width_span() {
    let source = Source::new("abc\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "test".to_string(),
        span: Span { start: 1, end: 1 },
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: test
 --> line 1, column 2
  |
1 | abc
  |  ^
",
    );
}

#[test]
fn caret_clamps_to_line_end() {
    let source = Source::new("abc\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "test".to_string(),
        span: Span { start: 1, end: 99 },
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: test
 --> line 1, column 2
  |
1 | abc
  |  ^^
",
    );
}

#[test]
fn span_starting_on_newline_renders_at_eol() {
    let source = Source::new("abc\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "test".to_string(),
        span: Span { start: 3, end: 3 }, // '\n'
        help: None,
        secondary: vec![],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: test
 --> line 1, column 4
  |
1 | abc
  |    ^
",
    );
}

#[test]
fn secondary_labels_do_not_shift_caret() {
    let source = Source::new("let x = y;\n".to_string());

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable".to_string(),
        span: Span { start: 8, end: 9 },
        help: None,
        secondary: vec![(Span { start: 4, end: 5 }, "defined here")],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: unknown variable
 --> line 1, column 9
  |
1 | let x = y;
  |         ^
  | -------- defined here
",
    );
}







