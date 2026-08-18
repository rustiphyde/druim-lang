use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process;

use druim::compiler::diagnostic::render;
use druim::compiler::error::{
    Diagnostic,
    Source,
    Span,
};
use druim::compiler::lexer::{LexError, Lexer};
use druim::compiler::parser::Parser;
use druim::compiler::semantics::eval::Evaluator;

fn main() {
    let exit_code = match run() {
        Ok(()) => 0,

        Err(message) => {
            eprintln!("{message}");
            1
        }
    };

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args();

    // Skip executable name.
    args.next();

    let Some(file_path) = args.next() else {
        return Err(
            "Druim expected a source file.\n\
             Usage: druim <file.drm>"
                .to_string(),
        );
    };

    if args.next().is_some() {
        return Err(
            "Druim accepts exactly one source file.\n\
             Usage: druim <file.drm>"
                .to_string(),
        );
    }

    let path = Path::new(&file_path);

    if path.extension().and_then(|extension| extension.to_str()) != Some("drm") {
        return Err(format!(
            "invalid Druim source file `{}`\n\
             Druim source files must use the `.drm` extension.",
            path.display(),
        ));
    }

    let source_text = fs::read_to_string(path)
        .map_err(|error| {
            match error.kind() {
                ErrorKind::NotFound => {
                    format!(
                        "Druim source file not found: `{}`",
                        path.display(),
                    )
                }

                ErrorKind::PermissionDenied => {
                    format!(
                        "Druim does not have permission to read `{}`",
                        path.display(),
                    )
                }

                ErrorKind::InvalidData => {
                    format!(
                        "Druim could not read `{}` as valid UTF-8 text.",
                        path.display(),
                    )
                }

                _ => {
                    format!(
                        "Druim could not read `{}`: {error}",
                        path.display(),
                    )
                }
            }
        })?;

    let source = Source::new(source_text.clone());

    let tokens = Lexer::new(&source_text)
        .tokenize()
        .map_err(|error| {
            render_lex_error(error, &source)
        })?;

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_file()
        .map_err(|diagnostic| {
            render(&diagnostic, &source)
        })?;

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .map_err(|diagnostic| {
            render(&diagnostic, &source)
        })?;

    Ok(())
}

fn render_lex_error(
    error: LexError,
    source: &Source,
) -> String {
    let diagnostic = match error {
        LexError::UnexpectedChar { ch, pos } => {
            Diagnostic::error(
                format!("unexpected character `{ch}`"),
                Span {
                    start: pos,
                    end: pos + ch.len_utf8(),
                },
            )
        }

        LexError::UnterminatedText { pos } => {
            Diagnostic::error(
                "unterminated text literal",
                Span {
                    start: pos,
                    end: pos + 1,
                },
            )
            .with_help(
                "Druim expected `\"` to close this text literal.",
            )
        }

        LexError::UnterminatedInterpolation { pos } => {
            Diagnostic::error(
                "unterminated text interpolation",
                Span {
                    start: pos,
                    end: pos + 2,
                },
            )
            .with_help(
                "Druim expected `.:` to close this text interpolation.",
            )
        }

        LexError::UnterminatedSingleComment { pos } => {
            Diagnostic::error(
                "unterminated single-line comment",
                Span {
                    start: pos,
                    end: pos + 2,
                },
            )
            .with_help(
                "Druim expected `-:` to close this single-line comment.",
            )
        }

        LexError::UnterminatedMultiComment { pos } => {
            Diagnostic::error(
                "unterminated multiline comment",
                Span {
                    start: pos,
                    end: pos + 3,
                },
            )
            .with_help(
                "Druim expected `--:` to close this multiline comment.",
            )
        }

        LexError::CommentInFunctionSyntax { pos } => {
            Diagnostic::error(
                "comment not allowed inside function parameters or arguments",
                Span {
                    start: pos,
                    end: pos + 2,
                },
            )
            .with_help(
                "Move the comment outside the function parameter or argument list.",
            )
        }
    };

    render(&diagnostic, source)
}