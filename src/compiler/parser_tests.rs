use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::ast::{Expr, Stmt, Literal};
use crate::compiler::diagnostic::render;
use crate::compiler::error::{Diagnostic, Source};


fn parse_stmt(src: &str) -> Stmt {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexing failed");
    let mut parser = Parser::new(&tokens);
    parser.parse_stmt().expect("failed to parse statement")
}

fn parse_expr_err(src: &str) -> Diagnostic {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexing failed");
    let mut parser = Parser::new(&tokens);

    parser
        .parse_expr()
        .expect_err("expected expression parse error")
}




#[test]
fn assign_from_statement() {
    let stmt = parse_stmt("x <- y;");

    assert_eq!(
        stmt,
        Stmt::AssignFrom {
            target: Expr::Ident("x".into()),
            source: Expr::Ident("y".into()),
        }
    );
}

#[test]
fn send_to_statement() {
    let stmt = parse_stmt("a -> b;");

    assert_eq!(
        stmt,
        Stmt::SendTo {
            value: Expr::Ident("a".into()),
            destination: Expr::Ident("b".into()),
        }
    );
}

#[test]
fn parses_multiple_statements() {
    let src = r#"
        a <- b;
        c -> d;
    "#;

    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexing failed");
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().expect("failed to parse program");

    assert_eq!(program.stmts.len(), 2);
}

#[test]
fn define_statement() {
    let src = "x = 42;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let stmt = parser.parse_stmt().expect("failed to parse define statement");

    match stmt {
        Stmt::Define { name, value } => {
            assert_eq!(name, "x");

            match value {
                Expr::Lit(Literal::Num(n)) => assert_eq!(n, 42),
                _ => panic!("expected numeric literal on right-hand side"),
            }
        }
        _ => panic!("expected Define statement"),
    }
}

#[test]
fn define_empty_statement() {
    let stmt = parse_stmt("a =;");

    assert_eq!(
        stmt,
        Stmt::DefineEmpty {
            name: "a".into()
        }
    );
}

#[test]
fn define_empty_requires_identifier_lhs() {
    let src = "(a) =;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_stmt().expect_err("expected invalid define-empty");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid define"),
        "expected invalid define error, got:\n{msg}"
    );
}

#[test]
fn define_empty_cannot_be_chained() {
    let src = "a =; = b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_program().expect_err("expected chained define-empty to fail");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid define"),
        "expected invalid define error, got:\n{msg}"
    );
}

#[test]
fn define_empty_is_not_expression() {
    let src = ":[ a =; ]:";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_expr().expect_err("expected expression error");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("expression"),
        "expected expression error, got:\n{msg}"
    );
}

#[test]
fn define_requires_identifier_lhs() {
    let src = "(x) = 1;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_stmt().expect_err("expected invalid define error");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid define statement"),
        "expected invalid define wording, got:\n{msg}"
    );

    assert!(
        msg.contains("define statements must start with an identifier"),
        "expected identifier-specific help, got:\n{msg}"
    );
}


#[test]
fn define_requires_semicolon() {
    let src = "x = 1";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_stmt().expect_err("expected missing semicolon error");
    let _ = err;
}

#[test]
fn define_cannot_be_chained() {
    let src = "a = b = c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_stmt().expect_err("expected chained define to fail");
    let _ = err;
}

#[test]
fn define_chaining_is_invalid_define() {
    let src = "a = b = c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_stmt().expect_err("expected chained define to fail");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid define statement"),
        "expected invalid define error, got:\n{msg}"
    );

    assert!(
        msg.contains("cannot be chained"),
        "expected chained-define help text, got:\n{msg}"
    );
}


#[test]
fn parses_statement_block() {
    let src = ":{ a <- b; c <- d; }:";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 1);

    match &program.stmts[0] {
        Stmt::Block { stmts } => {
            assert_eq!(stmts.len(), 2);

            matches!(stmts[0], Stmt::AssignFrom { .. });
            matches!(stmts[1], Stmt::AssignFrom { .. });
        }
        other => panic!("expected block statement, got {:?}", other),
    }
}

#[test]
fn parses_nested_statement_blocks() {
    let src = ":{ a <- b; :{ c <- d; }: }:";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();
    assert_eq!(program.stmts.len(), 1);

    match &program.stmts[0] {
        Stmt::Block { stmts } => {
            assert_eq!(stmts.len(), 2);

            // First statement: a <- b;
            matches!(stmts[0], Stmt::AssignFrom { .. });

            // Second statement: nested block
            match &stmts[1] {
                Stmt::Block { stmts: inner } => {
                    assert_eq!(inner.len(), 1);
                    matches!(inner[0], Stmt::AssignFrom { .. });
                }
                other => panic!("expected nested block, got {:?}", other),
            }
        }
        other => panic!("expected outer block, got {:?}", other),
    }
}

#[test]
fn block_requires_closing_delimiter() {
    let src = ":{ a <- b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_program().unwrap_err();

    let source = Source::new(src.to_string());
    let diag: Diagnostic = err.into();
    let msg = render(&diag, &source);
    assert!(msg.contains("}:"));
}

#[test]
fn parses_expression_block_literal() {
    let src = ":[ 42 ]:";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_expr().expect("failed to parse expression block");

    match expr {
        Expr::BlockExpr { expr: inner } => {
            match *inner {
                Expr::Lit(Literal::Num(n)) => assert_eq!(n, 42),
                other => panic!(
                    "expected numeric literal inside block expression, got {:?}",
                    other
                ),
            }
        }
        other => panic!("expected BlockExpr, got {:?}", other),
    }

}

#[test]
fn expression_block_respects_precedence() {

    let src = "1 + :[ 2 * 3 ]:";
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lex failed");

    let mut parser = Parser::new(&tokens);
    let expr = parser.parse_expr().expect("parse failed");

    match expr {
        Expr::Add(lhs, rhs) => {
            assert_eq!(*lhs, Expr::Lit(Literal::Num(1)));

            match *rhs {
                Expr::BlockExpr { expr } => match *expr {
                    Expr::Mul(a, b) => {
                        assert_eq!(*a, Expr::Lit(Literal::Num(2)));
                        assert_eq!(*b, Expr::Lit(Literal::Num(3)));
                    }
                    other => panic!("expected multiplication inside block, got {:?}", other),
                },
                other => panic!("expected block expression on RHS, got {:?}", other),
            }
        }
        other => panic!("expected addition at top level, got {:?}", other),
    }
}

#[test]
fn expression_block_rejects_statement() {
    let src = ":[ x = 3; ]:";

    let err = parse_expr_err(src);
    let source = Source::new(src.to_string());
    let diag: Diagnostic = err.into();
    let msg = render(&diag, &source);

    assert!(
        msg.contains("expression"),
        "expected expression error, got: {msg}"
    );
}

#[test]
fn parses_nested_expression_block() {
    let src = ":[ :[ 1 ]: ]:";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_expr().expect("failed to parse nested expression block");

    match expr {
        Expr::BlockExpr { expr: outer } => match *outer {
            Expr::BlockExpr { expr: inner } => match *inner {
                Expr::Lit(Literal::Num(n)) => assert_eq!(n, 1),
                other => panic!("expected numeric literal inside inner block, got {:?}", other),
            },
            other => panic!("expected inner BlockExpr, got {:?}", other),
        },
        other => panic!("expected outer BlockExpr, got {:?}", other),
    }
}

#[test]
fn nested_expression_block_respects_precedence() {
    let src = "1 + :[ 2 * :[ 3 + 4 ]: ]:";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_expr().expect("failed to parse nested precedence expression");

    match expr {
        Expr::Add(lhs, rhs) => {
            assert_eq!(*lhs, Expr::Lit(Literal::Num(1)));

            match *rhs {
                Expr::BlockExpr { expr } => match *expr {
                    Expr::Mul(a, b) => {
                        assert_eq!(*a, Expr::Lit(Literal::Num(2)));

                        match *b {
                            Expr::BlockExpr { expr } => match *expr {
                                Expr::Add(x, y) => {
                                    assert_eq!(*x, Expr::Lit(Literal::Num(3)));
                                    assert_eq!(*y, Expr::Lit(Literal::Num(4)));
                                }
                                other => panic!("expected addition inside inner block, got {:?}", other),
                            },
                            other => panic!("expected inner BlockExpr, got {:?}", other),
                        }
                    }
                    other => panic!("expected multiplication inside outer block, got {:?}", other),
                },
                other => panic!("expected BlockExpr on RHS, got {:?}", other),
            }
        }
        other => panic!("expected top-level addition, got {:?}", other),
    }
}

#[test]
fn bind_requires_identifier_lhs() {
    let src = ":= a;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_stmt().expect_err("expected invalid bind statement");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid bind statement"),
        "expected invalid statement error, got:\n{msg}"
    );

    assert!(
        msg.contains("identifier"),
        "expected identifier-related help text, got:\n{msg}"
    );
}

#[test]
fn guard_basic_statement() {
    let src = "x ?= y;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();
    assert_eq!(program.stmts.len(), 1);

    match &program.stmts[0] {
        Stmt::Guard { target, branches } => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 1);
        }
        _ => panic!("expected Guard statement"),
    }
}

#[test]
fn guard_single_fallback_statement() {


    let src = "x ?= y : z;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();
    assert_eq!(program.stmts.len(), 1);

    match &program.stmts[0] {
        Stmt::Guard { target, branches } => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 2);

            assert!(matches!(branches[0], Expr::Ident(ref s) if s == "y"));
            assert!(matches!(branches[1], Expr::Ident(ref s) if s == "z"));
        }
        other => panic!("expected Guard statement, got {:?}", other),
    }
}

#[test]
fn guard_chained_statement() {
    let src = "x ?= y : z : v : w;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.stmts.len(), 1);

    match &program.stmts[0] {
        Stmt::Guard { target, branches } => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 4);
        }
        _ => panic!("expected Guard statement"),
    }
}

#[test]
fn guard_requires_identifier_lhs() {
    let src = "?= a;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_stmt().expect_err("expected invalid guard statement");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid guard statement"),
        "expected invalid statement error, got:\n{msg}"
    );

    assert!(
        msg.contains("identifier"),
        "expected identifier-related help text, got:\n{msg}"
    );
}

#[test]
fn guard_allows_void_condition() {
    let src = "x ?= void;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let stmt = parser.parse_stmt().expect("expected guard statement to parse");

    match stmt {
        Stmt::Guard { target, branches } => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 1);

            match &branches[0] {
                Expr::Lit(Literal::Void) => {}
                other => panic!("expected void literal, got {:?}", other),
            }
        }
        other => panic!("expected Guard statement, got {:?}", other),
    }
}

#[test]
fn guard_rhs_cannot_be_empty() {
    let src = "a ?=;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_stmt()
        .expect_err("expected invalid guard statement");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid guard statement"),
        "expected guard-specific error, got:\n{msg}"
    );

    assert!(
        msg.contains("DefineEmpty"),
        "expected DefineEmpty suggestion, got:\n{msg}"
    );

    assert!(
        msg.contains("a =;"),
        "expected example syntax in help text, got:\n{msg}"
    );
}

#[test]
fn parses_function_with_expression_body() {
    let src = "fn add_one :( x )( x + 1 ):";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_expr().expect("failed to parse function");

    match expr {
        Expr::FnBlock { name, args, bodies } => {
            assert_eq!(name, "add_one");
            assert_eq!(args.len(), 1);
            assert_eq!(bodies.len(), 1);
        }
        other => panic!("expected FnBlock, got {:?}", other),
    }
}

#[test]
fn parses_return_statement_with_value() {
    let src = "ret 42;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let stmt = parser.parse_stmt().expect("failed to parse return");

    match stmt {
        Stmt::Return { value: Some(Expr::Lit(Literal::Num(n))) } => {
            assert_eq!(n, 42);
        }
        other => panic!("expected return statement, got {:?}", other),
    }
}












