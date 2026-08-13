use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::ast::{
    Bind, Block, Call, Copy, Define, DefineEmpty, Func, Guard, Literal, Node,
    NodeKind, Program, Ret,
};
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

// Empty Definition Tests
#[test]
fn parses_define_empty_node() {
    let node = parse_node("a =;");

    match &node.kind {
        NodeKind::DefineEmpty(DefineEmpty { name }) => {
            assert_eq!(name, "a");
        }

        other => panic!("expected DefineEmpty node, got {:?}", other),
    }
}

#[test]
fn define_empty_requires_identifier_lhs() {
    let src = "(a) =;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_node().expect_err("expected invalid empty definition error");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("invalid empty definition"),
        "expected invalid empty definition error, got:\n{msg}"
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
        msg.contains("invalid empty definition"),
        "expected invalid empty definition error, got:\n{msg}"
    );
}

#[test]
fn parses_local_define_empty_node() {
    let src = "loc a =;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().unwrap();

    match &(node).kind {
        NodeKind::Local(inner) => match &(inner.as_ref()).kind {
            NodeKind::DefineEmpty(DefineEmpty { name }) => {
                assert_eq!(name, "a");
            }
            other => panic!("expected empty definition inside local node, got {:?}", other),
        },
        other => panic!("expected local empty definition node, got {:?}", other),
    }
}

#[test]
fn define_empty_allows_following_statement() {
    let src = "a =; b =;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let first = parser.parse_node().unwrap();
    let second = parser.parse_node().unwrap();

    match &(first).kind {
        NodeKind::DefineEmpty(DefineEmpty { name }) => {
            assert_eq!(name, "a");
        }
        other => panic!("expected first empty definition node, got {:?}", other),
    }

    match &(second).kind {
        NodeKind::DefineEmpty(DefineEmpty { name }) => {
            assert_eq!(name, "b");
        }
        other => panic!("expected second empty definition node, got {:?}", other),
    }
}

#[test]
fn define_empty_rejects_repeated_local_modifier() {
    let src = "loc loc a =;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

// Define Tests
#[test]
fn parses_define_node() {
    let src = "x = 42;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().expect("failed to parse define node");

    match &(node).kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Lit(Literal::Num(n)) => assert_eq!(*n, 42),
                _ => panic!("expected numeric literal on right-hand side"),
            }
        }
        _ => panic!("expected Define node"),
    }
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
        msg.contains("begin with an identifier"),
        "expected identifier-specific help, got:\n{msg}"
    );
}

#[test]
fn define_requires_rhs_value() {
    let src = "a = ;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn define_rejects_single_identifier_rhs() {
    let src = "a = b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn define_rejects_extra_tokens_before_semicolon() {
    let src = "a = 12 13;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
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
fn define_cannot_chain_into_other_assignment_operator() {
    let src = "a = 12 :> b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn parses_local_define_node() {
    let src = "loc a = 12;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().unwrap();

    match &(node).kind {
        NodeKind::Local(inner) => match &(inner.as_ref()).kind {
            NodeKind::Define(Define { name, value }) => {
                assert_eq!(name, "a");

                match &(value.as_ref()).kind {
                    NodeKind::Lit(Literal::Num(value)) => {
                        assert_eq!(*value, 12);
                    }
                    other => panic!("expected numeric literal, got {:?}", other),
                }
            }
            other => panic!("expected define inside local node, got {:?}", other),
        },
        other => panic!("expected local define node, got {:?}", other),
    }
}

#[test]
fn define_rejects_repeated_local_modifier() {
    let src = "loc loc a = 12;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn define_accepts_compound_expression_rhs() {
    let src = "a = 12 + 13;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().unwrap();

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "a");

            match &value.kind {
                NodeKind::Add(lhs, rhs) => {
                    assert!(matches!(
                        &lhs.kind,
                        NodeKind::Lit(Literal::Num(12))
                    ));

                    assert!(matches!(
                        &rhs.kind,
                        NodeKind::Lit(Literal::Num(13))
                    ));
                }

                other => {
                    panic!("expected addition expression, got {:?}", other)
                }
            }
        }

        other => panic!("expected define node, got {:?}", other),
    }
}

#[test]
fn define_rejects_block_rhs() {
    let _ = parse_node_err(
        "result = :{ value = 10; }:",
    );
}

// Block Tests
#[test]
fn parses_node_block() {
    let src = ":{ a := b; c = 12; }:";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();

    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0].kind {
        NodeKind::Block(Block { segments }) => {
            assert_eq!(segments.len(), 1);
            assert_eq!(segments[0].nodes.len(), 2);

            assert!(matches!(&(segments[0].nodes[0]).kind,
                NodeKind::Copy(Copy { .. })
            ));

            assert!(matches!(&(segments[0].nodes[1]).kind,
                NodeKind::Define(Define { .. })
            ));
        }
        other => panic!("expected block node, got {:?}", other),
    }
}

#[test]
fn block_requires_closing_delimiter() {
    let src = ":{ a := b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser.parse_program().unwrap_err();

    let source = Source::new(src.to_string());
    let diag: Diagnostic = err;
    let msg = render(&diag, &source);
    assert!(msg.contains("Druim expected a closing block delimiter `}:`."));
}

// Copy Tests
#[test]
fn parses_copy_node() {
    let src = "a := b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().unwrap();

    match &(node).kind {
        NodeKind::Copy(Copy { name, target }) => {
            assert_eq!(name, "a");
            assert_eq!(target, "b");
        }
        other => panic!("expected copy node, got {:?}", other),
    }
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
fn copy_requires_identifier_rhs() {
    let src = "a := 12;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn copy_cannot_be_chained() {
    let src = "a := b := c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn copy_cannot_chain_into_other_assignment_operator() {
    let src = "a := b :> c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn copy_requires_semicolon() {
    let src = "a := b";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn parses_local_copy_node() {
    let src = "loc a := b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().unwrap();

    match &(node).kind {
        NodeKind::Local(inner) => match &(inner.as_ref()).kind {
            NodeKind::Copy(Copy { name, target }) => {
                assert_eq!(name, "a");
                assert_eq!(target, "b");
            }
            other => panic!("expected copy inside local node, got {:?}", other),
        },
        other => panic!("expected local copy node, got {:?}", other),
    }
}

#[test]
fn copy_rejects_extra_tokens_before_semicolon() {
    let src = "a := b c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}


// Bind Tests
#[test]
fn parses_bind_node() {
    let src = "a :> b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().unwrap();

    match &(node).kind {
        NodeKind::Bind(Bind { name, target, .. }) => {
            assert_eq!(name, "a");
            assert_eq!(target, "b");
        }
        other => panic!("expected bind node, got {:?}", other),
    }
}

#[test]
fn bind_requires_identifier_lhs() {
    let src = "12 :> b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let result = parser.parse_node();

    assert!(result.is_err());
}

#[test]
fn bind_requires_identifier_rhs() {
    let src = "a :> 12;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let result = parser.parse_node();

    assert!(result.is_err());
}

#[test]
fn bind_cannot_be_chained() {
    let src = "a :> b :> c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let result = parser.parse_node();

    assert!(result.is_err());
}

#[test]
fn bind_cannot_chain_into_other_assignment_operator() {
    let src = "a :> b := c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn bind_requires_semicolon() {
    let src = "a :> b";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let result = parser.parse_node();

    assert!(result.is_err());
}

#[test]
fn parses_local_bind_node() {
    let src = "loc a :> b;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().unwrap();

    match &(node).kind {
        NodeKind::Local(inner) => match &(inner.as_ref()).kind {
            NodeKind::Bind(Bind { name, target }) => {
                assert_eq!(name, "a");
                assert_eq!(target, "b");
            }
            other => panic!("expected local bind node, got {:?}", other),
        },
        other => panic!("expected local node, got {:?}", other),
    }
}

#[test]
fn bind_rejects_extra_tokens_before_semicolon() {
    let src = "a :> b c;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

// Guard Tests
#[test]
fn guard_basic_node() {
    let src = "x ?= y;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let program = parser.parse_program().unwrap();
    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0].kind {
        NodeKind::Guard(Guard { target, branches })  => {
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

    match &program.nodes[0].kind {
        NodeKind::Guard(Guard { target, branches })  => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 2);

            assert!(matches!(&branches[0].expr.kind,
                NodeKind::Ident(s) if s == "y"
            ));

            assert!(matches!(
                &branches[1].expr.kind,
                NodeKind::Ident(s) if s == "z"
            ));
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

    match &program.nodes[0].kind {
        NodeKind::Guard(Guard { target, branches })  => {
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

    match &(node).kind {
        NodeKind::Guard(Guard { target, branches }) => {
            assert_eq!(target, "x");
            assert_eq!(branches.len(), 1);

            match &branches[0].expr.kind {
                NodeKind::Lit(Literal::Void) => {}
                other => panic!("expected void branch, got {:?}", other),
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
        msg.contains("x =;"),
        "expected example syntax in help text, got:\n{msg}"
    );
}

#[test]
fn parses_return_node_with_value() {
    let node = parse_node("fn value :()(ret 42;):");

    match &node.kind {
        NodeKind::Func(Func { body, .. }) => {
            assert_eq!(body.len(), 1);

            match &body[0].kind {
                NodeKind::Ret(Ret { value: Some(value) }) => {
                    assert!(matches!(
                        &value.kind,
                        NodeKind::Lit(Literal::Num(42))
                    ));
                }

                other => panic!("expected return node, got {:?}", other),
            }
        }

        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_local_guard_node() {
    let src = "loc x ?= 12 : 13;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let node = parser.parse_node().unwrap();

    match &node.kind {
        NodeKind::Local(inner) => match &inner.kind {
            NodeKind::Guard(Guard { target, branches }) => {
                assert_eq!(target, "x");
                assert_eq!(branches.len(), 2);

                assert!(matches!(
                    &branches[0].expr.kind,
                    NodeKind::Lit(Literal::Num(12))
                ));

                assert!(matches!(
                    &branches[1].expr.kind,
                    NodeKind::Lit(Literal::Num(13))
                ));
            }

            other => panic!(
                "expected guard inside local node, got {:?}",
                other
            ),
        },

        other => panic!("expected local guard node, got {:?}", other),
    }
}

#[test]
fn guard_rejects_repeated_local_modifier() {
    let src = "loc loc x ?= 12;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn guard_rejects_extra_tokens_before_semicolon() {
    let src = "x ?= 12 13;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn guard_rejects_extra_tokens_after_final_branch() {
    let src = "x ?= 12 : 13 14;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn guard_cannot_chain_into_other_assignment_operator() {
    let src = "x ?= 12 := y;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

#[test]
fn guard_rejects_empty_later_branch() {
    let src = "x ?= 12 : ;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    assert!(parser.parse_node().is_err());
}

// Function Tests
#[test]
fn parses_function_with_single_param_and_body() {
    let src = "fn f :(x)(ret x;):";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_node().expect("failed to parse function");

    match &(expr).kind {
        NodeKind::Func(Func { name, params, body }) => {
            assert_eq!(name, "f");

            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "x");
            assert!(params[0].default.is_none());

            assert_eq!(body.len(), 1);

            match &body[0].kind {
                NodeKind::Ret(Ret {
                    value: Some(value),
                }) => {
                    assert!(matches!(&(value.as_ref()).kind,
                        NodeKind::Ident(s) if s == "x"
                    ));
                }
                other => panic!("expected `ret x;`, got {:?}", other),
            }
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

#[test]
fn parses_function_call_with_no_arguments() {
    let src = "f()";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser
        .parse_expr()
        .expect("failed to parse function call");

    match &(expr).kind {
        NodeKind::Call(Call { callee, args }) => {
            assert!(matches!(&(callee.as_ref()).kind,
                NodeKind::Ident(name) if name == "f"
            ));

            assert!(args.is_empty());
        }

        other => panic!("expected Call node, got {:?}", other),
    }
}

#[test]
fn parses_function_call_with_multiple_arguments() {
    let src = "f(a, b, c)";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser
        .parse_expr()
        .expect("failed to parse function call");

    match &(expr).kind {
        NodeKind::Call(Call { callee, args }) => {
            assert!(matches!(&(callee.as_ref()).kind,
                NodeKind::Ident(name) if name == "f"
            ));

            assert_eq!(args.len(), 3);

            assert!(matches!(
                &args[0].kind,
                NodeKind::Ident(name) if name == "a"
            ));
            assert!(matches!(
                &args[1].kind,
                NodeKind::Ident(name) if name == "b"
            ));
            assert!(matches!(
                &args[2].kind,
                NodeKind::Ident(name) if name == "c"
            ));
        }

        other => panic!("expected Call node, got {:?}", other),
    }
}

#[test]
fn parses_function_call_with_expression_arguments() {
    let src = "f(a + b, c * d)";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser
        .parse_expr()
        .expect("failed to parse function call");

    match &(expr).kind {
        NodeKind::Call(Call { callee, args }) => {
            assert!(matches!(&(callee.as_ref()).kind,
                NodeKind::Ident(name) if name == "f"
            ));

            assert_eq!(args.len(), 2);

            assert!(matches!(
                &args[0].kind,
                NodeKind::Add(_, _)
            ));
            assert!(matches!(
                &args[1].kind,
                NodeKind::Mul(_, _)
            ));
        }

        other => panic!("expected Call node, got {:?}", other),
    }
}

#[test]
fn function_calls_bind_tighter_than_addition() {
    let src = "f(a) + g(b)";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser
        .parse_expr()
        .expect("failed to parse expression");

    match &(expr).kind {
        NodeKind::Add(lhs, rhs) => {
            assert!(matches!(&(lhs.as_ref()).kind, NodeKind::Call(_)));
            assert!(matches!(&(rhs.as_ref()).kind, NodeKind::Call(_)));
        }

        other => panic!("expected Add node, got {:?}", other),
    }
}


#[test]
fn parses_nested_function_call_argument() {
    let src = "f(g(a))";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser
        .parse_expr()
        .expect("failed to parse nested function call");

    match &expr.kind {
        NodeKind::Call(Call { callee, args }) => {
            assert!(matches!(
                &callee.kind,
                NodeKind::Ident(name) if name == "f"
            ));

            assert_eq!(args.len(), 1);

            match &args[0].kind {
                 NodeKind::Call(Call {
                    callee: inner_callee,
                    args: inner_args,
                }) => {
                    assert!(matches!(
                        &inner_callee.kind,
                        NodeKind::Ident(name) if name == "g"
                    ));

                    assert_eq!(inner_args.len(), 1);

                    assert!(matches!(
                        &inner_args[0].kind,
                        NodeKind::Ident(name) if name == "a"
                    ));

                }

                other => {
                    panic!(
                        "expected nested Call argument, got {:?}",
                        other
                    )
                }
            }
        }

        other => panic!("expected outer Call node, got {:?}", other),
    }
}

#[test]
fn function_call_trailing_comma_is_error() {
    let src = "f(a,)";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_expr()
        .expect_err("expected trailing comma to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("argument")
            || rendered.contains("expression")
            || rendered.contains(")"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn function_call_requires_commas_between_arguments() {
    let src = "f(a b)";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_expr()
        .expect_err("expected missing comma to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("comma")
            || rendered.contains("separated")
            || rendered.contains("invalid function call"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn function_call_rejects_empty_first_argument() {
    let src = "f(, a)";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_expr()
        .expect_err("expected empty first argument to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("expression")
            || rendered.contains("argument")
            || rendered.contains("invalid function call"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn function_call_rejects_empty_middle_argument() {
    let src = "f(a,, b)";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_expr()
        .expect_err("expected empty middle argument to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("expression")
            || rendered.contains("argument")
            || rendered.contains("invalid function call"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn function_call_requires_closing_parenthesis() {
    let src = "f(a";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_expr()
        .expect_err("expected missing closing parenthesis to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains(")")
            || rendered.contains("closed")
            || rendered.contains("invalid function call"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn parses_function_call_with_single_argument() {
    let src = "f(a)";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser
        .parse_expr()
        .expect("failed to parse function call");

    match &(expr).kind {
        NodeKind::Call(Call { callee, args }) => {
            assert!(matches!(&(callee.as_ref()).kind,
                NodeKind::Ident(name) if name == "f"
            ));

            assert_eq!(args.len(), 1);

            assert!(matches!(&args[0].kind,
                NodeKind::Ident(name) if name == "a"
            ));
        }

        other => panic!("expected Call node, got {:?}", other),
    }
}

#[test]
fn parses_get_expression() {
    let node = parse_node("x = user::profile;");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Get(target, selector) => {
                    assert!(matches!(
                        &target.kind,
                        NodeKind::Ident(name) if name == "user"
                    ));

                    assert!(matches!(
                        &selector.kind,
                        NodeKind::Ident(name) if name == "profile"
                    ));
                }

                other => panic!("expected Get expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_chained_get_expression() {
    let node = parse_node("x = user::profile::email;");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Get(lhs, rhs) => {
                    assert!(matches!(
                        &rhs.kind,
                        NodeKind::Ident(name) if name == "email"
                    ));

                    match &lhs.kind {
                        NodeKind::Get(inner_lhs, inner_rhs) => {
                            assert!(matches!(
                                &inner_lhs.kind,
                                NodeKind::Ident(name) if name == "user"
                            ));

                            assert!(matches!(
                                &inner_rhs.kind,
                                NodeKind::Ident(name) if name == "profile"
                            ));
                        }

                        other => {
                            panic!("expected inner Get node, got {:?}", other)
                        }
                    }
                }

                other => panic!("expected outer Get node, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_has_expression() {
    let node = parse_node("x = user:?profile;");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Has(target, selector) => {
                    assert!(matches!(
                        &target.kind,
                        NodeKind::Ident(name) if name == "user"
                    ));

                    assert!(matches!(
                        &selector.kind,
                        NodeKind::Ident(name) if name == "profile"
                    ));
                }

                other => panic!("expected Has expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_has_at_end_of_get_chain() {
    let node = parse_node("x = user::profile:?email;");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Has(target, selector) => {
                    assert!(matches!(
                        &selector.kind,
                        NodeKind::Ident(name) if name == "email"
                    ));

                    match &target.kind {
                        NodeKind::Get(inner_target, inner_selector) => {
                            assert!(matches!(
                                &inner_target.kind,
                                NodeKind::Ident(name) if name == "user"
                            ));

                            assert!(matches!(
                                &inner_selector.kind,
                                NodeKind::Ident(name) if name == "profile"
                            ));
                        }

                        other => {
                            panic!("expected Get expression, got {:?}", other)
                        }
                    }
                }

                other => panic!("expected Has expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn rejects_get_after_has() {
    let src = "x = user:?profile::email;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected Get-after-Has error");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("cannot continue traversal after `:?`"),
        "expected Get-after-Has error, got:\n{msg}"
    );
}

#[test]
fn rejects_has_after_has() {
    let src = "x = user:?profile:?email;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected Has-after-Has error");

    let source = Source::new(src.to_string());
    let msg = render(&err, &source);

    assert!(
        msg.contains("cannot continue traversal after `:?`"),
        "expected Has-after-Has error, got:\n{msg}"
    );
}

#[test]
fn parses_empty_box_literal() {
    let src = "x = :[]:;";
    let node = parse_node(src);

    match &(node).kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &(value.as_ref()).kind {
                NodeKind::Box(box_literal) => {
                    assert!(box_literal.values.is_empty());
                }
                other => panic!("expected Box literal, got {:?}", other),
            }
        }
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_box_literal_with_values() {
    let src = "x = :[1, 2, 3]:;";
    let node = parse_node(src);

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Box(box_literal) => {
                    assert_eq!(box_literal.values.len(), 3);

                    assert!(matches!(
                        &box_literal.values[0].kind,
                        NodeKind::Lit(Literal::Num(1))
                    ));

                    assert!(matches!(
                        &box_literal.values[1].kind,
                        NodeKind::Lit(Literal::Num(2))
                    ));

                    assert!(matches!(
                        &box_literal.values[2].kind,
                        NodeKind::Lit(Literal::Num(3))
                    ));
                }

                other => panic!("expected Box literal, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_empty_bag_literal() {
    let src = "x = :||:;";
    let node = parse_node(src);

    match &(node).kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &(value.as_ref()).kind {
                NodeKind::Bag(bag_literal) => {
                    assert!(bag_literal.entries.is_empty());
                }
                other => panic!("expected Bag literal, got {:?}", other),
            }
        }
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_bag_literal_with_entries() {
    let src = r#"x = :| name: "Rusty", level: 42 |:;"#;
    let node = parse_node(src);

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Bag(bag_literal) => {
                    assert_eq!(bag_literal.entries.len(), 2);

                    assert_eq!(bag_literal.entries[0].name, "name");
                    assert!(matches!(
                        &bag_literal.entries[0].value.kind,
                        NodeKind::Lit(Literal::Text(value))
                            if value == "Rusty"
                    ));

                    assert_eq!(bag_literal.entries[1].name, "level");
                    assert!(matches!(
                        &bag_literal.entries[1].value.kind,
                        NodeKind::Lit(Literal::Num(42))
                    ));
                }

                other => panic!("expected Bag literal, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_box_inside_box() {
    let node = parse_node("x = :[1, :[2, 3]:, 4]:;");

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => {
            match &(value.as_ref()).kind {
                NodeKind::Box(outer) => {
                    assert_eq!(outer.values.len(), 3);
                    assert!(matches!(&outer.values[1].kind, NodeKind::Box(_)));
                }
                other => panic!("expected outer Box, got {:?}", other),
            }
        }
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_bag_inside_box() {
    let node = parse_node(
        r#"x = :[1, :| name: "Rusty" |:, 2]:;"#,
    );

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => {
            match &(value.as_ref()).kind {
                NodeKind::Box(outer) => {
                    assert_eq!(outer.values.len(), 3);
                    assert!(matches!(&outer.values[1].kind, NodeKind::Bag(_)));
                }
                other => panic!("expected outer Box, got {:?}", other),
            }
        }
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_box_inside_bag() {
    let node = parse_node(
        "x = :| items: :[1, 2, 3]: |:;",
    );

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => {
            match &(value.as_ref()).kind {
                NodeKind::Bag(bag) => {
                    assert_eq!(bag.entries.len(), 1);
                    assert!(matches!(&bag.entries[0].value.kind, NodeKind::Box(_)));
                }
                other => panic!("expected Bag, got {:?}", other),
            }
        }
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_bag_inside_bag() {
    let node = parse_node(
        r#"x = :| user: :| name: "Rusty" |: |:;"#,
    );

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => {
            match &(value.as_ref()).kind {
                NodeKind::Bag(bag) => {
                    assert_eq!(bag.entries.len(), 1);
                    assert!(matches!(&bag.entries[0].value.kind, NodeKind::Bag(_)));
                }
                other => panic!("expected outer Bag, got {:?}", other),
            }
        }
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_box_as_return_value() {
    let node = parse_node(
        "fn items :()(ret :[1, 2, 3]:;):",
    );

    match &node.kind {
        NodeKind::Func(Func { body, .. }) => {
            assert_eq!(body.len(), 1);

            match &body[0].kind {
                NodeKind::Ret(Ret { value: Some(value) }) => {
                    match &value.kind {
                        NodeKind::Box(box_literal) => {
                            assert_eq!(box_literal.values.len(), 3);
                        }

                        other => panic!(
                            "expected Box return value, got {:?}",
                            other
                        ),
                    }
                }

                other => panic!("expected return node, got {:?}", other),
            }
        }

        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_bag_as_return_value() {
    let node = parse_node(
        r#"fn player :()(ret :| name: "Rusty", level: 42 |:;):"#,
    );

    match &node.kind {
        NodeKind::Func(Func { body, .. }) => {
            assert_eq!(body.len(), 1);

            match &body[0].kind {
                NodeKind::Ret(Ret { value: Some(value) }) => {
                    match &value.kind {
                        NodeKind::Bag(bag_literal) => {
                            assert_eq!(bag_literal.entries.len(), 2);
                        }

                        other => panic!(
                            "expected Bag return value, got {:?}",
                            other
                        ),
                    }
                }

                other => panic!("expected return node, got {:?}", other),
            }
        }

        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_box_as_function_argument() {
    let src = "consume(:[1, 2, 3]:)";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_expr().expect("failed to parse function call");

    match &(expr).kind {
        NodeKind::Call(Call { args, .. }) => {
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0].kind, NodeKind::Box(_)));
        }
        other => panic!("expected Call node, got {:?}", other),
    }
}

#[test]
fn parses_bag_as_function_argument() {
    let src = r#"consume(:| name: "Rusty", level: 42 |:)"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_expr().expect("failed to parse function call");

    match &(expr).kind {
        NodeKind::Call(Call { args, .. }) => {
            assert_eq!(args.len(), 1);
            assert!(matches!(&args[0].kind, NodeKind::Bag(_)));
        }
        other => panic!("expected Call node, got {:?}", other),
    }
}

#[test]
fn parses_box_and_bag_as_multiple_function_arguments() {
    let src = r#"consume(:[1, 2]:, :| name: "Rusty" |:)"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let expr = parser.parse_expr().expect("failed to parse function call");

    match &(expr).kind {
        NodeKind::Call(Call { args, .. }) => {
            assert_eq!(args.len(), 2);
            assert!(matches!(&args[0].kind, NodeKind::Box(_)));
            assert!(matches!(&args[1].kind, NodeKind::Bag(_)));
        }
        other => panic!("expected Call node, got {:?}", other),
    }
}

#[test]
fn parses_box_as_default_parameter_value() {
    let src = "fn process :(items = :[1, 2, 3]:)(ret items;):";
    let node = parse_node(src);

    match &(node).kind {
        NodeKind::Func(Func { params, .. }) => {
            assert_eq!(params.len(), 1);

            match &params[0].default {
                Some(Node { kind: NodeKind::Box(box_literal), .. }) => {
                    assert_eq!(box_literal.values.len(), 3);
                }
                other => panic!(
                    "expected Box default parameter value, got {:?}",
                    other
                ),
            }
        }
        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_bag_as_default_parameter_value() {
    let src =
        r#"fn process :(options = :| active: true, retries: 3 |:)(ret options;):"#;

    let node = parse_node(src);

    match &(node).kind {
        NodeKind::Func(Func { params, .. }) => {
            assert_eq!(params.len(), 1);

            match &params[0].default {
                Some(Node { kind: NodeKind::Bag(bag_literal), .. }) => {
                    assert_eq!(bag_literal.entries.len(), 2);
                }
                other => panic!(
                    "expected Bag default parameter value, got {:?}",
                    other
                ),
            }
        }
        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_box_as_guard_branch() {
    let node = parse_node("result ?= :[1, 2, 3]:;");

    match &(node).kind {
        NodeKind::Guard(Guard { branches, .. }) => {
            assert_eq!(branches.len(), 1);
            assert!(matches!(&branches[0].expr.kind, NodeKind::Box(_)));
        }
        other => panic!("expected Guard node, got {:?}", other),
    }
}

#[test]
fn parses_bag_as_guard_branch() {
    let node = parse_node(
        r#"result ?= :| name: "Rusty", level: 42 |:;"#,
    );

    match &(node).kind {
        NodeKind::Guard(Guard { branches, .. }) => {
            assert_eq!(branches.len(), 1);
            assert!(matches!(&branches[0].expr.kind, NodeKind::Bag(_)));
        }
        other => panic!("expected Guard node, got {:?}", other),
    }
}

#[test]
fn parses_box_and_bag_as_guard_fallbacks() {
    let node = parse_node(
        r#"result ?= load() : :[1, 2, 3]: : :| name: "Rusty" |:;"#,
    );

    match &(node).kind {
        NodeKind::Guard(Guard { branches, .. }) => {
            assert_eq!(branches.len(), 3);

            assert!(matches!(&branches[0].expr.kind, NodeKind::Call(_)));
            assert!(matches!(&branches[1].expr.kind, NodeKind::Box(_)));
            assert!(matches!(&branches[2].expr.kind, NodeKind::Bag(_)));
        }
        other => panic!("expected Guard node, got {:?}", other),
    }
}

#[test]
fn box_rejects_missing_comma() {
    let src = "x = :[1 2]:;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected missing Box comma to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("missing comma in Box literal"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn box_rejects_trailing_comma() {
    let src = "x = :[1, 2,]:;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected trailing Box comma to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("trailing comma in Box literal"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn box_rejects_leading_comma() {
    let src = "x = :[, 1]:;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected leading Box comma to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("missing Box value"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn box_rejects_consecutive_commas() {
    let src = "x = :[1,, 2]:;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected consecutive Box commas to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("missing Box value"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn box_rejects_semicolon_separator() {
    let src = "x = :[1; 2]:;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected Box semicolon separator to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("invalid separator in Box literal"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn bag_rejects_missing_comma() {
    let src = r#"x = :| name: "Rusty" level: 42 |:;"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected missing Bag comma to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("missing comma in Bag literal"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn bag_rejects_trailing_comma() {
    let src = r#"x = :| name: "Rusty", |:;"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected trailing Bag comma to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("trailing comma in Bag literal"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn bag_rejects_leading_comma() {
    let src = r#"x = :|, name: "Rusty" |:;"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected leading Bag comma to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("missing Bag entry"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn bag_rejects_consecutive_commas() {
    let src = r#"x = :| name: "Rusty",, level: 42 |:;"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected consecutive Bag commas to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("missing Bag entry"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn bag_rejects_semicolon_separator() {
    let src = r#"x = :| name: "Rusty"; level: 42 |:;"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected Bag semicolon separator to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("invalid separator in Bag literal"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn bag_rejects_duplicate_entry_names() {
    let src = r#"x = :| name: "Rusty", name: "Atlas" |:;"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected duplicate Bag name to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("duplicate Bag entry name"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn bag_rejects_missing_entry_value() {
    let src = "x = :| name: |:;";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected missing Bag value to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("missing Bag entry value"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn bag_rejects_missing_colon_after_name() {
    let src = r#"x = :| name "Rusty" |:;"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected missing Bag colon to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("expected `:`")
            || rendered.contains("unexpected token"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn box_rejects_missing_closing_delimiter() {
    let src = ":[1, 2, 3";
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_expr()
        .expect_err("expected unterminated Box to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("unterminated Box literal"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn bag_rejects_missing_closing_delimiter() {
    let src = r#"x = :| name: "Rusty";"#;
    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected unterminated Bag to fail");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("unterminated Bag literal")
            || rendered.contains("invalid separator in Bag literal"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn parses_box_as_local_define_value() {
    let node = parse_node("loc items = :[1, 2, 3]:;");

    match &(node).kind {
        NodeKind::Local(inner) => match &(inner.as_ref()).kind {
            NodeKind::Define(Define { name, value }) => {
                assert_eq!(name, "items");
                assert!(matches!(&(value.as_ref()).kind, NodeKind::Box(_)));
            }
            other => panic!("expected Define inside Local, got {:?}", other),
        },
        other => panic!("expected Local node, got {:?}", other),
    }
}

#[test]
fn parses_bag_as_local_define_value() {
    let node = parse_node(
        r#"loc player = :| name: "Rusty", level: 42 |:;"#,
    );

    match &(node).kind {
        NodeKind::Local(inner) => match &(inner.as_ref()).kind {
            NodeKind::Define(Define { name, value }) => {
                assert_eq!(name, "player");
                assert!(matches!(&(value.as_ref()).kind, NodeKind::Bag(_)));
            }
            other => panic!("expected Define inside Local, got {:?}", other),
        },
        other => panic!("expected Local node, got {:?}", other),
    }
}

#[test]
fn box_accepts_mixed_value_types() {
    let node = parse_node(
        r#"x = :[
            42,
            3.14,
            "Rusty",
            void,
            true,
            false
        ]:;"#,
    );

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => match &(value.as_ref()).kind {
            NodeKind::Box(box_literal) => {
                assert_eq!(box_literal.values.len(), 6);

                assert!(matches!(&box_literal.values[0].kind,
                    NodeKind::Lit(Literal::Num(42))
                ));

                assert!(matches!(&box_literal.values[1].kind,
                    NodeKind::Lit(Literal::Dec(value)) if value == "3.14"
                ));

                assert!(matches!(&box_literal.values[2].kind,
                    NodeKind::Lit(Literal::Text(value)) if value == "Rusty"
                ));

                assert!(matches!(&box_literal.values[3].kind,
                    NodeKind::Lit(Literal::Void)
                ));

                assert!(matches!(&box_literal.values[4].kind,
                    NodeKind::Lit(Literal::Flag(true))
                ));

                assert!(matches!(&box_literal.values[5].kind,
                    NodeKind::Lit(Literal::Flag(false))
                ));
            }
            other => panic!("expected Box literal, got {:?}", other),
        },
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn bag_accepts_mixed_value_types() {
    let node = parse_node(
        r#"x = :|
            count: 42,
            ratio: 3.14,
            name: "Rusty",
            missing: void,
            active: true,
            enabled: false
        |:;"#,
    );

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => match &(value.as_ref()).kind {
            NodeKind::Bag(bag_literal) => {
                assert_eq!(bag_literal.entries.len(), 6);

                assert!(matches!(&bag_literal.entries[0].value.kind,
                    NodeKind::Lit(Literal::Num(42))
                ));

                assert!(matches!(&bag_literal.entries[1].value.kind,
                    NodeKind::Lit(Literal::Dec(value)) if value == "3.14"
                ));

                assert!(matches!(&bag_literal.entries[2].value.kind,
                    NodeKind::Lit(Literal::Text(value)) if value == "Rusty"
                ));

                assert!(matches!(&bag_literal.entries[3].value.kind,
                    NodeKind::Lit(Literal::Void)
                ));

                assert!(matches!(&bag_literal.entries[4].value.kind,
                    NodeKind::Lit(Literal::Flag(true))
                ));

                assert!(matches!(&bag_literal.entries[5].value.kind,
                    NodeKind::Lit(Literal::Flag(false))
                ));
            }
            other => panic!("expected Bag literal, got {:?}", other),
        },
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn box_accepts_expression_values() {
    let node = parse_node(
        r#"x = :[
            1 + 2,
            6 * 7,
            load()
        ]:;"#,
    );

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => match &(value.as_ref()).kind {
            NodeKind::Box(box_literal) => {
                assert_eq!(box_literal.values.len(), 3);

                assert!(matches!(&box_literal.values[0].kind,
                    NodeKind::Add(_, _)
                ));

                assert!(matches!(&box_literal.values[1].kind,
                    NodeKind::Mul(_, _)
                ));

                assert!(matches!(&box_literal.values[2].kind,
                    NodeKind::Call(_)
                ));
            }
            other => panic!("expected Box literal, got {:?}", other),
        },
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn bag_accepts_expression_values() {
    let node = parse_node(
        r#"x = :|
            total: 1 + 2,
            product: 6 * 7,
            result: load()
        |:;"#,
    );

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => match &(value.as_ref()).kind {
            NodeKind::Bag(bag_literal) => {
                assert_eq!(bag_literal.entries.len(), 3);

                assert!(matches!(&bag_literal.entries[0].value.kind,
                    NodeKind::Add(_, _)
                ));

                assert!(matches!(&bag_literal.entries[1].value.kind,
                    NodeKind::Mul(_, _)
                ));

                assert!(matches!(&bag_literal.entries[2].value.kind,
                    NodeKind::Call(_)
                ));
            }
            other => panic!("expected Bag literal, got {:?}", other),
        },
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn box_accepts_identifier_values() {
    let node = parse_node("x = :[first, second, third]:;");

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => match &(value.as_ref()).kind {
            NodeKind::Box(box_literal) => {
                assert_eq!(box_literal.values.len(), 3);

                assert!(matches!(&box_literal.values[0].kind,
                    NodeKind::Ident(name) if name == "first"
                ));

                assert!(matches!(&box_literal.values[1].kind,
                    NodeKind::Ident(name) if name == "second"
                ));

                assert!(matches!(&box_literal.values[2].kind,
                    NodeKind::Ident(name) if name == "third"
                ));
            }
            other => panic!("expected Box literal, got {:?}", other),
        },
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn bag_accepts_identifier_values() {
    let node = parse_node(
        "x = :| primary: first, secondary: second |:;",
    );

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => match &(value.as_ref()).kind {
            NodeKind::Bag(bag_literal) => {
                assert_eq!(bag_literal.entries.len(), 2);

                assert!(matches!(&bag_literal.entries[0].value.kind,
                    NodeKind::Ident(name) if name == "first"
                ));

                assert!(matches!(&bag_literal.entries[1].value.kind,
                    NodeKind::Ident(name) if name == "second"
                ));
            }
            other => panic!("expected Bag literal, got {:?}", other),
        },
        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_program_with_box_and_bag_definitions() {
    let program = parse_program(
        r#"
            items = :[1, 2, 3]:;
            player = :| name: "Rusty", level: 42 |:;
        "#,
    );

    assert_eq!(program.nodes.len(), 2);

    assert!(matches!(&program.nodes[0].kind,
        NodeKind::Define(Define { value, .. })
            if matches!(&(value.as_ref()).kind, NodeKind::Box(_))
    ));

    assert!(matches!(&program.nodes[1].kind,
        NodeKind::Define(Define { value, .. })
            if matches!(&(value.as_ref()).kind, NodeKind::Bag(_))
    ));
}

#[test]
fn parses_program_with_nested_collections_and_following_statement() {
    let program = parse_program(
        r#"
            data = :[
                :| name: "Rusty" |:,
                :[1, 2, 3]:
            ]:;

            next = 42;
        "#,
    );

    assert_eq!(program.nodes.len(), 2);

    assert!(matches!(&program.nodes[0].kind,
        NodeKind::Define(Define { value, .. })
            if matches!(&(value.as_ref()).kind, NodeKind::Box(_))
    ));

    assert!(matches!(&program.nodes[1].kind,
        NodeKind::Define(Define { value, .. })
            if matches!(&(value.as_ref()).kind,
                NodeKind::Lit(Literal::Num(42))
            )
    ));
}

#[test]
fn parses_box_definition_inside_block() {
    let program = parse_program(
        r#"
            :{
                items = :[1, 2, 3]:;
            }:
        "#,
    );

    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0].kind {
        NodeKind::Block(Block { segments }) => {
            assert_eq!(segments.len(), 1);
            assert_eq!(segments[0].nodes.len(), 1);

            assert!(matches!(&segments[0].nodes[0].kind,
                NodeKind::Define(Define { value, .. })
                    if matches!(&(value.as_ref()).kind, NodeKind::Box(_))
            ));
        }

        other => panic!("expected Block node, got {:?}", other),
    }
}

#[test]
fn parses_bag_definition_inside_block() {
    let program = parse_program(
        r#"
            :{
                player = :|
                    name: "Rusty",
                    level: 42
                |:;
            }:
        "#,
    );

    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0].kind {
        NodeKind::Block(Block { segments }) => {
            assert_eq!(segments.len(), 1);
            assert_eq!(segments[0].nodes.len(), 1);

            assert!(matches!(&segments[0].nodes[0].kind,
                NodeKind::Define(Define { value, .. })
                    if matches!(&(value.as_ref()).kind, NodeKind::Bag(_))
            ));
        }

        other => panic!("expected Block node, got {:?}", other),
    }
}

#[test]
fn parses_box_definition_inside_function_body() {
    let node = parse_node(
        r#"
            fn build_items :()(
                items = :[1, 2, 3]:;
                ret items;
            ):
        "#,
    );

    match &(node).kind {
        NodeKind::Func(Func { body, .. }) => {
            assert_eq!(body.len(), 2);

            assert!(matches!(&body[0].kind,
                NodeKind::Define(Define { value, .. })
                    if matches!(&(value.as_ref()).kind, NodeKind::Box(_))
            ));

            assert!(matches!(&body[1].kind,
                NodeKind::Ret(Ret { .. })
            ));
        }

        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_bag_definition_inside_function_body() {
    let node = parse_node(
        r#"
            fn build_player :()(
                player = :|
                    name: "Rusty",
                    level: 42
                |:;
                ret player;
            ):
        "#,
    );

    match &(node).kind {
        NodeKind::Func(Func { body, .. }) => {
            assert_eq!(body.len(), 2);

            assert!(matches!(&body[0].kind,
                NodeKind::Define(Define { value, .. })
                    if matches!(&(value.as_ref()).kind, NodeKind::Bag(_))
            ));

            assert!(matches!(&body[1].kind,
                NodeKind::Ret(Ret { .. })
            ));
        }

        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_empty_box_and_bag_inside_box() {
    let node = parse_node("x = :[:[]:, :||:]:;");

    match &(node).kind {
        NodeKind::Define(Define { value, .. }) => match &(value.as_ref()).kind {
            NodeKind::Box(box_literal) => {
                assert_eq!(box_literal.values.len(), 2);

                assert!(matches!(&box_literal.values[0].kind,
                    NodeKind::Box(inner) if inner.values.is_empty()
                ));

                assert!(matches!(&box_literal.values[1].kind,
                    NodeKind::Bag(inner) if inner.entries.is_empty()
                ));
            }

            other => panic!("expected outer Box literal, got {:?}", other),
        },

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_true_flag_literal() {
    let node = parse_node("x = true;");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            assert!(matches!(
                &value.kind,
                NodeKind::Lit(Literal::Flag(true))
            ));
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_false_flag_literal() {
    let node = parse_node("x = false;");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            assert!(matches!(
                &value.kind,
                NodeKind::Lit(Literal::Flag(false))
            ));
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_indexed_get_expression() {
    let node = parse_node("x = items::[1];");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Get(target, selector) => {
                    assert!(matches!(
                        &target.kind,
                        NodeKind::Ident(name) if name == "items"
                    ));

                    match &selector.kind {
                        NodeKind::Index(index) => {
                            assert!(matches!(
                                &index.kind,
                                NodeKind::Lit(Literal::Num(1))
                            ));
                        }

                        other => panic!(
                            "expected indexed selector, got {:?}",
                            other
                        ),
                    }
                }

                other => panic!("expected Get expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_indexed_has_expression() {
    let node = parse_node("x = items:?[1];");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Has(target, selector) => {
                    assert!(matches!(
                        &target.kind,
                        NodeKind::Ident(name) if name == "items"
                    ));

                    match &selector.kind {
                        NodeKind::Index(index) => {
                            assert!(matches!(
                                &index.kind,
                                NodeKind::Lit(Literal::Num(1))
                            ));
                        }

                        other => panic!(
                            "expected indexed selector, got {:?}",
                            other
                        ),
                    }
                }

                other => panic!("expected Has expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn rejects_unclosed_index_selector() {
    let src = "x = items::[0;";

    let err = parse_node_err(src);

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("expected `]` after Box index"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn parses_computed_index_expression() {
    let node = parse_node("x = items::[1 + 1];");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Get(target, selector) => {
                    assert!(matches!(
                        &target.kind,
                        NodeKind::Ident(name) if name == "items"
                    ));

                    match &selector.kind {
                        NodeKind::Index(index_expr) => {
                            match &index_expr.kind {
                                NodeKind::Add(lhs, rhs) => {
                                    assert!(matches!(
                                        &lhs.kind,
                                        NodeKind::Lit(Literal::Num(1))
                                    ));

                                    assert!(matches!(
                                        &rhs.kind,
                                        NodeKind::Lit(Literal::Num(1))
                                    ));
                                }

                                other => {
                                    panic!(
                                        "expected Add expression inside Index, got {:?}",
                                        other
                                    )
                                }
                            }
                        }

                        other => {
                            panic!("expected Index selector, got {:?}", other)
                        }
                    }
                }

                other => panic!("expected Get expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_repeated_indexed_get_expression() {
    let node = parse_node("x = matrix::[1]::[0];");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Get(target, selector) => {
                    match &target.kind {
                        NodeKind::Get(inner_target, inner_selector) => {
                            assert!(matches!(
                                &inner_target.kind,
                                NodeKind::Ident(name) if name == "matrix"
                            ));

                            match &inner_selector.kind {
                                NodeKind::Index(index) => {
                                    assert!(matches!(
                                        &index.kind,
                                        NodeKind::Lit(Literal::Num(1))
                                    ));
                                }

                                other => panic!(
                                    "expected first indexed selector, got {:?}",
                                    other
                                ),
                            }
                        }

                        other => panic!(
                            "expected inner Get expression, got {:?}",
                            other
                        ),
                    }

                    match &selector.kind {
                        NodeKind::Index(index) => {
                            assert!(matches!(
                                &index.kind,
                                NodeKind::Lit(Literal::Num(0))
                            ));
                        }

                        other => panic!(
                            "expected second indexed selector, got {:?}",
                            other
                        ),
                    }
                }

                other => panic!("expected outer Get expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_indexed_get_followed_by_named_get() {
    let node = parse_node("x = data::[0]::name;");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Get(target, selector) => {
                    assert!(matches!(
                        &selector.kind,
                        NodeKind::Ident(name) if name == "name"
                    ));

                    match &target.kind {
                        NodeKind::Get(inner_target, inner_selector) => {
                            assert!(matches!(
                                &inner_target.kind,
                                NodeKind::Ident(name) if name == "data"
                            ));

                            match &inner_selector.kind {
                                NodeKind::Index(index) => {
                                    assert!(matches!(
                                        &index.kind,
                                        NodeKind::Lit(Literal::Num(0))
                                    ));
                                }

                                other => panic!(
                                    "expected indexed selector, got {:?}",
                                    other
                                ),
                            }
                        }

                        other => panic!(
                            "expected inner Get expression, got {:?}",
                            other
                        ),
                    }
                }

                other => panic!("expected outer Get expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_named_get_followed_by_indexed_get() {
    let node = parse_node("x = data::items::[1];");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Get(target, selector) => {
                    match &target.kind {
                        NodeKind::Get(inner_target, inner_selector) => {
                            assert!(matches!(
                                &inner_target.kind,
                                NodeKind::Ident(name) if name == "data"
                            ));

                            assert!(matches!(
                                &inner_selector.kind,
                                NodeKind::Ident(name) if name == "items"
                            ));
                        }

                        other => panic!(
                            "expected inner Get expression, got {:?}",
                            other
                        ),
                    }

                    match &selector.kind {
                        NodeKind::Index(index) => {
                            assert!(matches!(
                                &index.kind,
                                NodeKind::Lit(Literal::Num(1))
                            ));
                        }

                        other => panic!(
                            "expected indexed selector, got {:?}",
                            other
                        ),
                    }
                }

                other => panic!("expected outer Get expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_indexed_get_followed_by_named_has() {
    let node = parse_node("x = data::[0]:?name;");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Has(target, selector) => {
                    assert!(matches!(
                        &selector.kind,
                        NodeKind::Ident(name) if name == "name"
                    ));

                    match &target.kind {
                        NodeKind::Get(inner_target, inner_selector) => {
                            assert!(matches!(
                                &inner_target.kind,
                                NodeKind::Ident(name) if name == "data"
                            ));

                            match &inner_selector.kind {
                                NodeKind::Index(index) => {
                                    assert!(matches!(
                                        &index.kind,
                                        NodeKind::Lit(Literal::Num(0))
                                    ));
                                }

                                other => panic!(
                                    "expected indexed selector, got {:?}",
                                    other
                                ),
                            }
                        }

                        other => panic!(
                            "expected Get expression before Has, got {:?}",
                            other
                        ),
                    }
                }

                other => panic!("expected Has expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_named_get_followed_by_indexed_has() {
    let node = parse_node("x = data::items:?[1];");

    match &node.kind {
        NodeKind::Define(Define { name, value }) => {
            assert_eq!(name, "x");

            match &value.kind {
                NodeKind::Has(target, selector) => {
                    match &target.kind {
                        NodeKind::Get(inner_target, inner_selector) => {
                            assert!(matches!(
                                &inner_target.kind,
                                NodeKind::Ident(name) if name == "data"
                            ));

                            assert!(matches!(
                                &inner_selector.kind,
                                NodeKind::Ident(name) if name == "items"
                            ));
                        }

                        other => panic!(
                            "expected inner Get expression, got {:?}",
                            other
                        ),
                    }

                    match &selector.kind {
                        NodeKind::Index(index) => {
                            assert!(matches!(
                                &index.kind,
                                NodeKind::Lit(Literal::Num(1))
                            ));
                        }

                        other => panic!(
                            "expected indexed selector, got {:?}",
                            other
                        ),
                    }
                }

                other => panic!("expected Has expression, got {:?}", other),
            }
        }

        other => panic!("expected Define node, got {:?}", other),
    }
}

#[test]
fn parses_basic_loop() {
    let node = parse_node(
        r#"
        :<
            index = 0;
            limit = 3;
        >?<
            index < limit
        >?<
            index = index + 1;
        >:
        "#,
    );

    match &(node).kind {
        NodeKind::Loop(loop_node) => {
            assert_eq!(loop_node.setup.len(), 2);

            assert!(matches!(
                &loop_node.condition.kind,
                NodeKind::Lt(lhs, rhs)
                    if matches!(
                        &lhs.kind,
                        NodeKind::Ident(name) if name == "index"
                    )
                    && matches!(
                        &rhs.kind,
                        NodeKind::Ident(name) if name == "limit"
                    )
            ));

            assert_eq!(loop_node.process.len(), 1);

            assert!(matches!(
                &loop_node.process[0].kind,
                NodeKind::Define(Define { name, value })
                    if name == "index"
                        && matches!(
                            &value.kind,
                            NodeKind::Add(lhs, rhs)
                                if matches!(
                                    &lhs.kind,
                                    NodeKind::Ident(name) if name == "index"
                                )
                                && matches!(
                                    &rhs.kind,
                                    NodeKind::Lit(Literal::Num(1))
                                )
                        )
            ));
        }

        other => panic!("expected loop node, got {:?}", other),
    }
}

#[test]
fn parses_nested_loop() {
    let node = parse_node(
        r#"
        :<
            outer_index = 0;
            outer_limit = 2;
        >?<
            outer_index < outer_limit
        >?<
            :<
                inner_index = 0;
                inner_limit = 3;
            >?<
                inner_index < inner_limit
            >?<
                inner_index = inner_index + 1;
            >:

            outer_index = outer_index + 1;
        >:
        "#,
    );

    match &node.kind {
        NodeKind::Loop(outer_loop) => {
            assert_eq!(outer_loop.setup.len(), 2);
            assert_eq!(outer_loop.process.len(), 2);

            match &outer_loop.process[0].kind {
                NodeKind::Loop(inner_loop) => {
                    assert_eq!(inner_loop.setup.len(), 2);
                    assert_eq!(inner_loop.process.len(), 1);

                    assert!(matches!(
                        &inner_loop.condition.kind,
                        NodeKind::Lt(lhs, rhs)
                            if matches!(
                                &lhs.kind,
                                NodeKind::Ident(name)
                                    if name == "inner_index"
                            )
                            && matches!(
                                &rhs.kind,
                                NodeKind::Ident(name)
                                    if name == "inner_limit"
                            )
                    ));
                }

                other => {
                    panic!("expected nested loop node, got {:?}", other)
                }
            }

            assert!(matches!(
                &outer_loop.process[1].kind,
                NodeKind::Define(Define { name, value })
                    if name == "outer_index"
                        && matches!(
                            &value.kind,
                            NodeKind::Add(lhs, rhs)
                                if matches!(
                                    &lhs.kind,
                                    NodeKind::Ident(name)
                                        if name == "outer_index"
                                )
                                && matches!(
                                    &rhs.kind,
                                    NodeKind::Lit(Literal::Num(1))
                                )
                        )
            ));
        }

        other => panic!("expected outer loop node, got {:?}", other),
    }
}

#[test]
fn loop_requires_first_separator() {
    let _ = parse_node_err(
        r#"
        :<
            index = 0;
            limit = 3;

            index < limit
        >?<
            index = index + 1;
        >:
        "#,
    );
}

#[test]
fn loop_requires_second_separator() {
    let _ = parse_node_err(
        r#"
        :<
            index = 0;
            limit = 3;
        >?<
            index < limit

            index = index + 1;
        >:
        "#,
    );
}

#[test]
fn loop_requires_closing_delimiter() {
    let _ = parse_node_err(
        r#"
        :<
            index = 0;
            limit = 3;
        >?<
            index < limit
        >?<
            index = index + 1;
        "#,
    );
}

#[test]
fn define_rejects_loop_rhs() {
    let _ = parse_node_err(
        r#"
        result = :<
            index = 0;
        >?<
            index < 3
        >?<
            index = index + 1;
        >:;
        "#,
    );
}

#[test]
fn parses_loop_inside_function_body() {
    let src = r#"
        fn repeat :()(
            :<
                index = 0;
            >?< index < 3
            >?<
                index = index + 1;
            >:

            ret 7;
        ):
    "#;

    let node = parse_node(src);

    match &(node).kind {
        NodeKind::Func(Func { name, params, body }) => {
            assert_eq!(name, "repeat");
            assert!(params.is_empty());
            assert_eq!(body.len(), 2);

            match &body[0].kind {
                NodeKind::Loop(loop_node) => {
                    assert_eq!(loop_node.setup.len(), 1);
                    assert_eq!(loop_node.process.len(), 1);

                    assert!(matches!(&(loop_node.setup[0]).kind,
                        NodeKind::Define(_)
                    ));

                    assert!(matches!(&(loop_node.condition.as_ref()).kind,
                        NodeKind::Lt(_, _)
                    ));

                    assert!(matches!(&(loop_node.process[0]).kind,
                        NodeKind::Define(_)
                    ));
                }

                other => {
                    panic!(
                        "expected loop as first function-body node, got {:?}",
                        other,
                    );
                }
            }

            assert!(matches!(&body[1].kind,
                NodeKind::Ret(Ret {
                    value: Some(value),
                }) if matches!(&(value.as_ref()).kind,
                    NodeKind::Lit(Literal::Num(7))
                )
            ));
        }

        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_function_definition_inside_block() {
    let program = parse_program(
        r#"
            :{
                fn build :()(
                    ret 42;
                ):
            }:
        "#,
    );

    assert_eq!(program.nodes.len(), 1);

    match &program.nodes[0].kind {
        NodeKind::Block(Block { segments }) => {
            assert_eq!(segments.len(), 1);
            assert_eq!(segments[0].nodes.len(), 1);

            assert!(matches!(
                &segments[0].nodes[0].kind,
                NodeKind::Func(Func { name, .. }) if name == "build"
            ));
        }

        other => panic!("expected Block node, got {:?}", other),
    }
}

#[test]
fn parses_multiple_default_parameters() {
    let node = parse_node(
        "fn configure :(width = 10, height = 20)(ret width + height;):",
    );

    match &node.kind {
        NodeKind::Func(Func { params, .. }) => {
            assert_eq!(params.len(), 2);

            assert_eq!(params[0].name, "width");
            assert!(matches!(
                &params[0].default,
                Some(Node {
                    kind: NodeKind::Lit(Literal::Num(10)),
                    ..
                })
            ));

            assert_eq!(params[1].name, "height");
            assert!(matches!(
                &params[1].default,
                Some(Node {
                    kind: NodeKind::Lit(Literal::Num(20)),
                    ..
                })
            ));
        }

        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn parses_required_parameter_after_default_parameter() {
    let node = parse_node(
        "fn configure :(width = 10, height)(ret height;):",
    );

    match &node.kind {
        NodeKind::Func(Func { params, .. }) => {
            assert_eq!(params.len(), 2);

            assert_eq!(params[0].name, "width");
            assert!(params[0].default.is_some());

            assert_eq!(params[1].name, "height");
            assert!(params[1].default.is_none());
        }

        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn function_rejects_duplicate_parameter_names() {
    let src = "fn configure :(width, width = 10)(ret width;):";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected duplicate parameter names to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("duplicate function parameter"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn function_rejects_trailing_parameter_comma() {
    let src = "fn configure :(width,)(ret width;):";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected trailing parameter comma to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("invalid function parameter")
            || rendered.contains("parameter list"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn function_rejects_leading_parameter_comma() {
    let src = "fn configure :(, width)(ret width;):";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected leading parameter comma to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("invalid function parameter")
            || rendered.contains("parameter list"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn function_rejects_consecutive_parameter_commas() {
    let src = "fn configure :(width,, height)(ret width;):";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected consecutive parameter commas to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("invalid function parameter")
            || rendered.contains("parameter list"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn parses_function_with_empty_body() {
    let node = parse_node("fn noop :()():");

    match &node.kind {
        NodeKind::Func(Func { name, params, body }) => {
            assert_eq!(name, "noop");
            assert!(params.is_empty());
            assert!(body.is_empty());
        }

        other => panic!("expected Func node, got {:?}", other),
    }
}

#[test]
fn function_rejects_multiple_bodies() {
    let src = "fn noop :()()():";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected multiple function bodies to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("function chaining not allowed"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn return_is_rejected_outside_function() {
    let src = "ret 42;";

    let tokens = Lexer::new(src).tokenize().unwrap();
    let mut parser = Parser::new(&tokens);

    let err = parser
        .parse_node()
        .expect_err("expected top-level return to be rejected");

    let source = Source::new(src.to_string());
    let rendered = render(&err, &source);

    assert!(
        rendered.contains("return")
            && rendered.contains("function"),
        "unexpected error message:\n{}",
        rendered
    );
}

#[test]
fn parses_mutate_node() {
    let tokens = Lexer::new("count << count + 1;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let node = parser
        .parse_node()
        .expect("parsing should succeed");

    match node.kind {
        NodeKind::Mutate(mutate) => {
            assert_eq!(mutate.name, "count");
        }
        other => panic!("expected mutate node, got {other:?}"),
    }
}

#[test]
fn parses_local_mutate_node() {
    let tokens = Lexer::new("loc count << count + 1;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let node = parser
        .parse_node()
        .expect("parsing should succeed");

    match node.kind {
        NodeKind::Local(inner) => {
            assert!(matches!(inner.kind, NodeKind::Mutate(_)));
        }
        other => panic!("expected local mutate node, got {other:?}"),
    }
}

#[test]
fn mutate_requires_rhs_value() {
    let tokens = Lexer::new("count <<;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    assert!(
        parser.parse_node().is_err(),
        "mutate without a RHS must be rejected"
    );
}

#[test]
fn mutate_requires_semicolon() {
    let tokens = Lexer::new("count << count + 1")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    assert!(
        parser.parse_node().is_err(),
        "unterminated mutate must be rejected"
    );
}

#[test]
fn mutate_cannot_be_chained() {
    let tokens = Lexer::new("count << count + 1 << 2;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    assert!(
        parser.parse_node().is_err(),
        "mutate statements cannot be chained"
    );
}

#[test]
fn parses_stone_mutate_node() {
    let tokens = Lexer::new("stone count << count + 1;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);
    let node = parser.parse_node().expect("parsing should succeed");

    match node.kind {
        NodeKind::Stone(inner) => {
            assert!(matches!(inner.kind, NodeKind::Mutate(_)));
        }
        other => panic!("expected stone mutate node, got {other:?}"),
    }
}

#[test]
fn parses_stone_local_mutate_node() {
    let tokens = Lexer::new("stone loc count << count + 1;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);
    let node = parser.parse_node().expect("parsing should succeed");

    match node.kind {
        NodeKind::Stone(inner) => {
            assert!(matches!(inner.kind, NodeKind::Local(_)));
        }
        other => panic!("expected stone local mutate node, got {other:?}"),
    }
}

#[test]
fn parses_stone_global_mutate_node() {
    let tokens = Lexer::new("stone glo count << count + 1;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let node = parser
        .parse_node()
        .expect("parsing should succeed");

    match node.kind {
        NodeKind::Stone(inner) => {
            match inner.kind {
                NodeKind::Global(inner) => {
                    match inner.kind {
                        NodeKind::Mutate(mutate) => {
                            assert_eq!(mutate.name, "count");
                        }

                        other => {
                            panic!(
                                "expected mutate inside global, got {other:?}"
                            );
                        }
                    }
                }

                other => {
                    panic!(
                        "expected global inside stone, got {other:?}"
                    );
                }
            }
        }

        other => {
            panic!(
                "expected stone global mutate node, got {other:?}"
            );
        }
    }
}

#[test]
fn parses_stone_define_node() {
    let tokens = Lexer::new("stone value = 10;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);
    let node = parser.parse_node().expect("parsing should succeed");

    match node.kind {
        NodeKind::Stone(inner) => {
            assert!(matches!(inner.kind, NodeKind::Define(_)));
        }
        other => panic!("expected stone define node, got {other:?}"),
    }
}

#[test]
fn parses_global_define_node() {
    let tokens = Lexer::new("glo value = 10;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);
    let node = parser.parse_node().expect("parsing should succeed");

    match node.kind {
        NodeKind::Global(inner) => {
            assert!(matches!(inner.kind, NodeKind::Define(_)));
        }
        other => panic!("expected global define node, got {other:?}"),
    }
}

#[test]
fn parses_stone_global_define_node() {
    let tokens = Lexer::new("stone glo value = 10;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);
    let node = parser.parse_node().expect("parsing should succeed");

    match node.kind {
        NodeKind::Stone(inner) => {
            assert!(matches!(inner.kind, NodeKind::Global(_)));
        }
        other => panic!("expected stone global define node, got {other:?}"),
    }
}

#[test]
fn parses_stone_global_define_empty_node() {
    let tokens = Lexer::new("stone glo value =;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let node = parser
        .parse_node()
        .expect("parsing should succeed");

    match node.kind {
        NodeKind::Stone(inner) => {
            match inner.kind {
                NodeKind::Global(inner) => {
                    assert!(matches!(inner.kind, NodeKind::DefineEmpty(_)));
                }

                other => {
                    panic!("expected global inside stone, got {other:?}");
                }
            }
        }

        other => {
            panic!("expected stone global define-empty node, got {other:?}");
        }
    }
}

#[test]
fn parses_stone_global_copy_node() {
    let tokens = Lexer::new("stone glo snapshot := source;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let node = parser
        .parse_node()
        .expect("parsing should succeed");

    match node.kind {
        NodeKind::Stone(inner) => {
            match inner.kind {
                NodeKind::Global(inner) => {
                    match inner.kind {
                        NodeKind::Copy(copy) => {
                            assert_eq!(copy.name, "snapshot");
                            assert_eq!(copy.target, "source");
                        }

                        other => {
                            panic!("expected copy inside global, got {other:?}");
                        }
                    }
                }

                other => {
                    panic!("expected global inside stone, got {other:?}");
                }
            }
        }

        other => {
            panic!("expected stone global copy node, got {other:?}");
        }
    }
}

#[test]
fn parses_stone_global_bind_node() {
    let tokens = Lexer::new("stone glo alias :> source;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let node = parser
        .parse_node()
        .expect("parsing should succeed");

    match node.kind {
        NodeKind::Stone(inner) => {
            match inner.kind {
                NodeKind::Global(inner) => {
                    match inner.kind {
                        NodeKind::Bind(bind) => {
                            assert_eq!(bind.name, "alias");
                            assert_eq!(bind.target, "source");
                        }

                        other => {
                            panic!("expected bind inside global, got {other:?}");
                        }
                    }
                }

                other => {
                    panic!("expected global inside stone, got {other:?}");
                }
            }
        }

        other => {
            panic!("expected stone global bind node, got {other:?}");
        }
    }
}

#[test]
fn parses_stone_global_guard_node() {
    let tokens = Lexer::new("stone glo result ?= first : second;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let node = parser
        .parse_node()
        .expect("parsing should succeed");

    match node.kind {
        NodeKind::Stone(inner) => {
            match inner.kind {
                NodeKind::Global(inner) => {
                    match inner.kind {
                        NodeKind::Guard(guard) => {
                            assert_eq!(guard.target, "result");
                            assert_eq!(guard.branches.len(), 2);
                        }

                        other => {
                            panic!("expected guard inside global, got {other:?}");
                        }
                    }
                }

                other => {
                    panic!("expected global inside stone, got {other:?}");
                }
            }
        }

        other => {
            panic!("expected stone global guard node, got {other:?}");
        }
    }
}

#[test]
fn rejects_loc_stone_modifier_order() {
    let tokens = Lexer::new("loc stone value = 10;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    assert!(
        parser.parse_node().is_err(),
        "`stone` must precede `loc`"
    );
}

#[test]
fn rejects_glo_stone_modifier_order() {
    let tokens = Lexer::new("glo stone value = 10;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    assert!(
        parser.parse_node().is_err(),
        "`stone` must precede `glo`"
    );
}

#[test]
fn rejects_loc_glo_combination() {
    let tokens = Lexer::new("loc glo value = 10;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    assert!(
        parser.parse_node().is_err(),
        "`loc` and `glo` cannot be combined"
    );
}

#[test]
fn rejects_glo_loc_combination() {
    let tokens = Lexer::new("glo loc value = 10;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    assert!(
        parser.parse_node().is_err(),
        "`glo` and `loc` cannot be combined"
    );
}

#[test]
fn rejects_repeated_stone_modifier() {
    let tokens = Lexer::new("stone stone value = 10;")
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    assert!(
        parser.parse_node().is_err(),
        "`stone` cannot be repeated"
    );
}