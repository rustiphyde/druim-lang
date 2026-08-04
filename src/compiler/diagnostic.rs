use crate::compiler::error::{Diagnostic, Note, Severity, Source, Span};

#[derive(Copy, Clone)]
enum Style {
    Error,
    Warning,
    Note,
    Help,
    Caret,
    Plain,
}

#[cfg(feature = "ansi")]
fn apply_ansi(style: Style, text: &str) -> String {
    if !ansi_enabled() {
        return text.to_string();
    }

    let code = match style {
        Style::Error => "\x1b[38;5;88m",
        Style::Warning => "\x1b[38;5;202m",
        Style::Note => "\x1b[38;5;94m",
        Style::Help => "\x1b[38;5;64m",
        Style::Caret => "\x1b[38;5;135m",
        Style::Plain => "",
    };

    if code.is_empty() {
        text.to_string()
    } else {
        format!("{code}{text}\x1b[39m")
    }
}


#[cfg(feature = "ansi")]
fn ansi_enabled() -> bool {
    std::env::var_os("DRUIM_ANSI").is_some()
}

#[cfg(not(feature = "ansi"))]
fn apply_ansi(_style: Style, text: &str) -> String {
    text.to_string()
}

fn write_styled(out: &mut String, style: Style, text: &str) {
    let rendered = apply_ansi(style, text);
    out.push_str(&rendered);
}

// Renders a source span using `span.start` as the authoritative
// position of the first caret.
// Secondary labels, notes, and other annotations must not alter it.
fn render_span_block(
    out: &mut String,
    source: &Source,
    span: Span,
) {
    let (line, col) = source.line_col(span.start);
    write_styled(
        out,
        Style::Plain,
        &format!(" --> line {}, column {}\n", line, col),
    );


    let line_text = source.line_text(line);
    let gutter_width = format!("{}", line).len();

    write_styled(
        out,
        Style::Plain,
        &format!("{:>width$} |\n", "", width = gutter_width),
    );

    write_styled(
        out,
        Style::Plain,
        &format!(
            "{:>width$} | {}\n",
            line,
            line_text,
            width = gutter_width
        ),
    );

    let line_len = line_text.chars().count();
    let span_starts_on_newline = source.is_newline_at(span.start);

    let start_col = if span_starts_on_newline {
        line_len
    } else {
        col.saturating_sub(1).min(line_len)
    };

    let width = source
        .span_char_width(span)
        .min(line_len.saturating_sub(start_col))
        .max(1);

    // Prefix: gutter + bar + spaces before caret (PLAIN)
    let mut prefix = format!("{:>width$} | ", "", width = gutter_width);

    for _ in 0..start_col {
        prefix.push(' ');
    }

    write_styled(out, Style::Plain, &prefix);

    // Caret run: ONLY the carets (STYLED)
    let carets = "^".repeat(width);
    write_styled(out, Style::Caret, &carets);

    // Newline (PLAIN)
    out.push('\n');
}

fn render_secondary_labels(
    out: &mut String,
    source: &Source,
    primary_span: Span,
    secondary: &[(Span, &'static str)],
) {
    if secondary.is_empty() {
        return;
    }

    let (primary_line, primary_col) = source.line_col(primary_span.start);
    let gutter_width = format!("{}", primary_line).len();

    let primary_line_text = source.line_text(primary_line);
    let primary_line_len = primary_line_text.chars().count();

    let primary_start_col = if source.is_newline_at(primary_span.start) {
        primary_line_len
    } else {
        primary_col
            .saturating_sub(1)
            .min(primary_line_len)
    };

    for (span, label) in secondary {
        let (secondary_line, secondary_col) = source.line_col(span.start);

        if secondary_line != primary_line {
            out.push('\n');
            write_styled(out, Style::Plain, label);
            out.push('\n');

            render_span_block(out, source, *span);
            continue;
        }

        let secondary_line_text = source.line_text(secondary_line);
        let secondary_line_len = secondary_line_text.chars().count();

        let secondary_start_col = if source.is_newline_at(span.start) {
            secondary_line_len
        } else {
            secondary_col
                .saturating_sub(1)
                .min(secondary_line_len)
        };

        // A connector must begin before the primary caret.
        if secondary_start_col >= primary_start_col {
            continue;
        }

        // Dashes end one column before the primary caret.
        // The following space places the label directly beneath it.
        let dash_len = primary_start_col
            .saturating_sub(secondary_start_col)
            .saturating_sub(1);

        let mut prefix =
            format!("{:>width$} | ", "", width = gutter_width);

        for _ in 0..secondary_start_col {
            prefix.push(' ');
        }

        write_styled(out, Style::Plain, &prefix);

        for _ in 0..dash_len {
            out.push('-');
        }

        out.push(' ');
        write_styled(out, Style::Plain, label);
        out.push('\n');
    }
}

fn render_note(out: &mut String, note: &Note, source: &Source) {
    let severity = match note.severity {
        Severity::Note => "note",
        Severity::Help => "help",
        Severity::Warning => "warning",
        Severity::Error => "error",
    };

    let style = match note.severity {
        Severity::Error => Style::Error,
        Severity::Warning => Style::Warning,
        Severity::Note => Style::Note,
        Severity::Help => Style::Help,
    };

    write_styled(
        out,
        style,
        &format!("{severity}: {}\n", note.message),
    );


    let span = match note.span {
        Some(s) => s,
        None => return,
    };

    render_span_block(out, source, span);
}

/// Render a diagnostic into a human-readable message.
/// This is the ONLY place where user-facing formatting occurs.
pub fn render(diagnostic: &Diagnostic, source: &Source) -> String {
    let mut out = String::new();

    let severity = match diagnostic.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
        Severity::Help => "help",
    };

    let style = match diagnostic.severity {
        Severity::Error => Style::Error,
        Severity::Warning => Style::Warning,
        Severity::Note => Style::Note,
        Severity::Help => Style::Help,
    };

    write_styled(
        &mut out,
        style,
        &format!("{severity}: {}\n", diagnostic.message),
    );

    // Top-level Note/Help diagnostics:
    // - If span is empty (start==end), do not render source.
    // - Otherwise, render source block.
    if matches!(diagnostic.severity, Severity::Note | Severity::Help) {
        if diagnostic.span.start == diagnostic.span.end {
            return out;
        }
        render_span_block(&mut out, source, diagnostic.span);
        return out;
    }

    // ----- Errors / Warnings only below -----

    render_span_block(&mut out, source, diagnostic.span);

    // Secondary labels (must render after the primary caret block)
    render_secondary_labels(
        &mut out,
        source,
        diagnostic.span,
        &diagnostic.secondary,
    );

    // Notes
    for note in &diagnostic.notes {
        out.push('\n');
        render_note(&mut out, note, source);
    }

    // Help (always last, always separated)
    if let Some(help) = diagnostic.help {
        out.push('\n');
        write_styled(
            &mut out, 
            Style::Plain, 
            &format!("help: {}\n", help)
        );
    }

    out
}
