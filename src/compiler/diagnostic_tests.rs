use crate::compiler::diagnostic::render;
use crate::compiler::error::{Diagnostic, Note, Severity, Source, Span};

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
    let source_text = "let x = ;\n";
    let source = Source::new(source_text.to_string());

    let semicolon_start = source_text
        .find(';')
        .expect("source should contain `;`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unexpected token".to_string(),
        span: Span {
            start: semicolon_start,
            end: semicolon_start + 1,
        },
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
    let source_text = "let total = 123;\n";
    let source = Source::new(source_text.to_string());

    let number_start = source_text
        .find("123")
        .expect("source should contain `123`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "invalid number".to_string(),
        span: Span {
            start: number_start,
            end: number_start + "123".len(),
        },
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

    let bad_start = source_text
        .find("bad")
        .expect("source should contain `bad`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "invalid syntax".to_string(),
        span: Span {
            start: bad_start,
            end: bad_start + "bad".len(),
        },
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
    let source_text = "let x = 1;\n";
    let source = Source::new(source_text.to_string());

    let x_start = source_text
        .find('x')
        .expect("source should contain `x`");

    let diag = Diagnostic {
        severity: Severity::Warning,
        message: "unused variable".to_string(),
        span: Span {
            start: x_start,
            end: x_start + 1,
        },
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
    let source_text = "oops = 1;\n";
    let source = Source::new(source_text.to_string());

    let oops_start = source_text
        .find("oops")
        .expect("source should contain `oops`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unexpected identifier".to_string(),
        span: Span {
            start: oops_start,
            end: oops_start + "oops".len(),
        },
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
    let source_text = "\
let a = 1;
let b = ;
let c = 3;
";
    let source = Source::new(source_text.to_string());

    let line_two_start = source_text
        .find("let b = ;")
        .expect("source should contain line two");

    let semicolon_start = line_two_start + "let b = ".len();

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "expected expression".to_string(),
        span: Span {
            start: semicolon_start,
            end: semicolon_start + 1,
        },
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
    let source_text = "\
let total = price * qty;
let price = 10;
";
    let source = Source::new(source_text.to_string());

    let qty_start = source_text
        .find("qty")
        .expect("source should contain `qty`");

    let price_start = source_text
        .find("price")
        .expect("source should contain `price`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable `qty`".to_string(),
        span: Span {
            start: qty_start,
            end: qty_start + "qty".len(),
        },
        help: None,
        secondary: vec![(
            Span {
                start: price_start,
                end: price_start + "price".len(),
            },
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
  |             ------- defined here
",
    );
}

#[test]
fn render_error_with_multiple_secondary_labels() {
    let source_text = "\
let total = price * qty + tax;
let price = 10;
let tax = 2;
";
    let source = Source::new(source_text.to_string());

    let primary_start = source_text
        .find("qty + tax")
        .expect("source should contain `qty + tax`");

    let price_start = source_text
        .find("price")
        .expect("source should contain the first `price`");

    let declared_tax_start = source_text
        .rfind("tax")
        .expect("source should contain the declared `tax`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variables".to_string(),
        span: Span {
            start: primary_start,
            end: primary_start + "qty + tax".len(),
        },
        help: None,
        secondary: vec![
            (
                Span {
                    start: price_start,
                    end: price_start + "price".len(),
                },
                "defined here",
            ),
            (
                Span {
                    start: declared_tax_start,
                    end: declared_tax_start + "tax".len(),
                },
                "defined here",
            ),
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
  |             ------- defined here

defined here
 --> line 3, column 5
  |
3 | let tax = 2;
  |     ^^^
",
    );
}

#[test]
fn render_error_with_note_and_help() {
    let source_text = "x = y;\n";
    let source = Source::new(source_text.to_string());

    let y_start = source_text
        .find('y')
        .expect("source should contain `y`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable `y`".to_string(),
        span: Span {
            start: y_start,
            end: y_start + 1,
        },
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
    let source_text = "\
let total = price * qty;
let price = 10;
";
    let source = Source::new(source_text.to_string());

    let qty_start = source_text
        .find("qty")
        .expect("source should contain `qty`");

    let price_start = source_text
        .find("price")
        .expect("source should contain `price`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable `qty`".to_string(),
        span: Span {
            start: qty_start,
            end: qty_start + "qty".len(),
        },
        help: None,
        secondary: vec![],
        notes: vec![
            Note {
                severity: Severity::Note,
                message: "`price` is defined here".to_string(),
                span: Some(Span {
                    start: price_start,
                    end: price_start + "price".len(),
                }),
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
    let source_text = "\
let total = price * qty;
let price = 10;
";
    let source = Source::new(source_text.to_string());

    let qty_start = source_text
        .find("qty")
        .expect("source should contain `qty`");

    let price_start = source_text
        .find("price")
        .expect("source should contain `price`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable `qty`".to_string(),
        span: Span {
            start: qty_start,
            end: qty_start + "qty".len(),
        },
        help: Some("declare `qty` before use"),
        secondary: vec![],
        notes: vec![
            Note {
                severity: Severity::Note,
                message: "`price` is defined here".to_string(),
                span: Some(Span {
                    start: price_start,
                    end: price_start + "price".len(),
                }),
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
    let source_text = "let x = y;\n";
    let source = Source::new(source_text.to_string());

    let x_start = source_text
        .find('x')
        .expect("source should contain `x`");

    let y_start = source_text
        .find('y')
        .expect("source should contain `y`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable".to_string(),
        span: Span {
            start: y_start,
            end: y_start + 1,
        },
        help: None,
        secondary: vec![(
            Span {
                start: x_start,
                end: x_start + 1,
            },
            "defined here",
        )],
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
  |     --- defined here
",
    );
}

#[test]
fn diagnostic_caret_uses_character_columns_for_utf8_source() {
    let source_text = "value = \"é\" + bad;\n".to_string();
    let source = Source::new(source_text.clone());

    let bad_start = source_text
        .find("bad")
        .expect("test source should contain `bad`");

    let diag = Diagnostic::error(
        "invalid value",
        Span {
            start: bad_start,
            end: bad_start + "bad".len(),
        },
    );

    assert_render(
        &diag,
        &source,
        "\
error: invalid value
 --> line 1, column 15
  |
1 | value = \"é\" + bad;
  |               ^^^
",
    );
}

#[test]
fn cross_line_secondary_span_renders_its_own_source_block() {
    let source_text = "\
let total = pirce * qty;
let price = 10;
";

    let source = Source::new(source_text.to_string());

    let primary_start = source_text
        .find("pirce")
        .expect("source should contain `pirce`");

    let secondary_start = source_text
        .rfind("price")
        .expect("source should contain `price`");

    let diag = Diagnostic {
        severity: Severity::Error,
        message: "unknown variable `pirce`".to_string(),
        span: Span {
            start: primary_start,
            end: primary_start + "pirce".len(),
        },
        help: None,
        secondary: vec![(
            Span {
                start: secondary_start,
                end: secondary_start + "price".len(),
            },
            "similar name defined here",
        )],
        notes: vec![],
    };

    assert_render(
        &diag,
        &source,
        "\
error: unknown variable `pirce`
 --> line 1, column 13
  |
1 | let total = pirce * qty;
  |             ^^^^^

similar name defined here
 --> line 2, column 5
  |
2 | let price = 10;
  |     ^^^^^
",
    );
}







