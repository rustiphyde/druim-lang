use std::env;
use std::fs;
use std::path::Path;
use std::process;

use druim::compiler::diagnostic::render;
use druim::compiler::error::Source;
use druim::compiler::lexer::Lexer;
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
            "usage: druim <file.drm>".to_string()
        );
    };

    if args.next().is_some() {
        return Err(
            "usage: druim <file.drm>".to_string()
        );
    }

    let path = Path::new(&file_path);

    if path.extension().and_then(|extension| extension.to_str()) != Some("drm") {
        return Err(
            "Druim source files must use the `.drm` extension.".to_string()
        );
    }

    let source_text = fs::read_to_string(path)
        .map_err(|error| {
            format!(
                "could not read `{}`: {error}",
                path.display()
            )
        })?;

    let source = Source::new(source_text.clone());

    let tokens = Lexer::new(&source_text)
        .tokenize()
        .map_err(|error| {
            format!("lexer error: {error:?}")
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