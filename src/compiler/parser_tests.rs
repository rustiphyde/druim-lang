use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::ast::{Node, Block, Define, DefineEmpty, Copy, Bind, Guard, Ret, Program, Func, Literal};
use crate::compiler::diagnostic::render;
use crate::compiler::error::{Diagnostic, Source};

fn parse_node(src: &str) -> Node {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexing failed");
    let mut parser = Parser::new(&tokens);
    parser.parse_node().expect("failed to parse node")
}

fn parse_node_err(src: &str) -> Diagnostic {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexing failed");
    let mut parser = Parser::new(&tokens);
    parser.parse_node().expect_err("expected parse error")
}

fn parse_program(src: &str) -> Program {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexing failed");
    let mut parser = Parser::new(&tokens);
    parser.parse_program().expect("failed to parse program")
}

#[test]
fn parses_multiple_nodes() {
    let src = r#"
        a = 12;
        c := a;
    "#;

    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("lexing failed");
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().expect("failed to parse program");

    assert_eq!(program.nodes.len(), 2);
}

#[test]
fn parses_define_node() {
    let src = "x = 42;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().expect("failed to parse define node");

    match node {
        Node::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match *value {
                Node::Lit(Literal::Num(n)) => assert_eq!(n, 42),
                _ => panic!("expected numeric literal on right-hand side"),
            }
        }
        _ => panic!("expected Define node"),
    }
}

#[test]
fn parses_define_empty_node() {
    let node = parse_node("a =;");

    assert_eq!(
        node,
        Node::DefineEmpty(DefineEmpty {
            name: "a".into()
        })
    );
}

#[test]
fn define_empty_requires_identifier_lhs() {
    let src = "(a) =;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_node().expect_err("expected invalid define-empty");

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
fn define_requires_identifier_lhs() {
    let src = "(x) = 1;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_node().expect_err("expected invalid define error");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid define"),
        "expected invalid define wording, got:\n{msg}"
    );

    assert!(
        msg.contains("expected identifier"),
        "expected identifier-specific help, got:\n{msg}"
    );
}


#[test]
fn define_requires_semicolon() {
    let src = "x = 1";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_node().expect_err("expected missing semicolon error");
    let _ = err;
}

#[test]
fn define_cannot_be_chained() {
    let src = "a = b = c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_node().expect_err("expected chained define to fail");
    let _ = err;
}

#[test]
fn define_chaining_is_invalid_define() {
    let src = "a = b = c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_node().expect_err("expected chained define to fail");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid define"),
        "expected invalid define error, got:\n{msg}"
    );

    assert!(
        msg.contains("cannot be chained"),
        "expected chained-define help text, got:\n{msg}"
    );
}


#[test]
fn parses_node_block() {
    let src = ":{ a := b; c = 12; }:";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();

    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0] {
        Node::Block(Block { nodes }) => {
            assert_eq!(nodes.len(), 2);

            matches!(nodes[0], Node::Copy(Copy{ .. }));
            matches!(nodes[1], Node::Define(Define { .. }));
        }
        other => panic!("expected block node, got {:?}", other),
    }
}

#[test]
fn parses_nested_node_blocks() {
    let src = ":{ a :> b; :{ c := d; }: }:";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();
    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0] {
        Node::Block(Block { nodes }) => {
            assert_eq!(nodes.len(), 2);

            // First node: a <- b;
            matches!(nodes[0], Node::Bind(Bind { .. }));

            // Second node: nested block
            match &nodes[1] {
                Node::Block(Block { nodes: inner }) => {
                    assert_eq!(inner.len(), 1);
                    matches!(inner[0], Node::Copy(Copy { .. }));
                }
                other => panic!("expected nested block, got {:?}", other),
            }
        }
        other => panic!("expected outer block, got {:?}", other),
    }
}

#[test]
fn block_requires_closing_delimiter() {
    let src = ":{ a := b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_program().unwrap_err();

    let source = Source::new(src.to_string());
    let diag: Diagnostic = err.into();
    let msg = render(&diag, &source);
    assert!(msg.contains("Druim expected a closing block delimiter `}:`."));
}

#[test]
fn copy_requires_identifier_lhs() {
    let src = ":= a;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_node().expect_err("expected invalid copy error");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid copy"),
        "expected invalid copy error, got:\n{msg}"
    );

    assert!(
        msg.contains("identifier"),
        "expected identifier-related help text, got:\n{msg}"
    );
}

#[test]
fn guard_basic_node() {
    let src = "x ?= y;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();
    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0] {
        Node::Guard(Guard { target, branches })  => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 1);
        }
        _ => panic!("expected guard"),
    }
}

#[test]
fn guard_single_fallback_node() {


    let src = "x ?= y : z;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();
    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0] {
        Node::Guard(Guard { target, branches })  => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 2);

            assert!(matches!(branches[0], Node::Ident(ref s) if s == "y"));
            assert!(matches!(branches[1], Node::Ident(ref s) if s == "z"));
        }
        other => panic!("expected guard, got {:?}", other),
    }
}

#[test]
fn guard_chained_node() {
    let src = "x ?= y : z : v : w;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0] {
        Node::Guard(Guard { target, branches })  => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 4);
        }
        _ => panic!("expected guard"),
    }
}

#[test]
fn guard_requires_identifier_lhs() {
    let src = "?= a;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_node().expect_err("expected invalid guard error");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid guard"),
        "expected invalid guard error, got:\n{msg}"
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

    let node = parser.parse_node().expect("expected guard node to parse");

    match node {
        Node::Guard(Guard { target, branches }) => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 1);

            match &branches[0] {
                Node::Lit(Literal::Void) => {}
                other => panic!("expected void literal, got {:?}", other),
            }
        }
        other => panic!("expected guard node, got {:?}", other),
    }
}

#[test]
fn guard_rhs_cannot_be_empty() {
    let src = "a ?=;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected invalid guard error");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid guard"),
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
fn parses_return_node_with_value() {
    let src = "ret 42;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().expect("failed to parse ret");

    match node {
        Node::Ret(Ret { value: Some(value) }) => {
            match *value {
                Node::Lit(Literal::Num(n)) => {
                    assert_eq!(n, 42);
                }
                other => panic!("expected numeric literal, got {:?}", other),
            }
        }
        other => panic!("expected ret node, got {:?}", other),
    }
}

#[test]
fn parses_function_with_single_param_and_body() {
    let src = "fn f :(x)(x):";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_node().expect("failed to parse function");

    match expr {
        Node::Func(Func { name, params, bodies }) => {
            assert_eq!(name, "f");

            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "x");
            assert!(params[0].default.is_none());

            assert_eq!(bodies.len(), 1);

            match &bodies[0] {
                Node::Ident(id) => assert_eq!(id, "x"),
                other => panic!("expected body node `x`, got {:?}", other),
            }
        }
        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_function_with_multiple_bodies() {
    let src = "fn f :(x)(x)(x + 1):";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_node().expect("failed to parse function");

    match expr {
        Node::Func(Func { bodies, .. }) => {
            assert_eq!(bodies.len(), 2);
        }
        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn function_missing_body_block_is_error() {
    let src = "fn f :(x):";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected error for missing body block");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("body"),
        "unexpected error message:\n{}",
        rendered
    );
}




