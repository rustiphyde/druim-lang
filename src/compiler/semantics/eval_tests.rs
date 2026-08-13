use crate::compiler::ast::{
    Guard, GuardBranch, Literal, Node, NodeKind,
};
use crate::compiler::error::Span;
use crate::compiler::semantics::eval::Evaluator;
use crate::compiler::semantics::value::Value;


fn test_span() -> Span {
    Span { start: 0, end: 0 }
}

fn branch(v: Literal) -> GuardBranch {
    GuardBranch {
        expr: Node::new(NodeKind::Lit(v), test_span()),
    }
}

#[test]
fn guard_assigns_first_truthy_branch() {
    let node = Node::new(NodeKind::Guard(Guard {
        target: "x".into(),
        branches: vec![
            branch(Literal::Flag(false)),
            branch(Literal::Num(1)),
            branch(Literal::Num(2)),
        ],
    }), test_span());

    let mut ev = Evaluator::new();
    ev.eval_node(&node)
    .expect("node evaluation should succeed");

    match ev.get("x") {
        Some(Value::Num(n)) => assert_eq!(n, 1),
        other => panic!("expected x = Num(1), got {:?}", other),
    }
}

#[test]
fn guard_skips_false_values_until_true() {
    let node = Node::new(NodeKind::Guard(Guard {
        target: "x".into(),
        branches: vec![
            branch(Literal::Void),
            branch(Literal::Num(0)),
            branch(Literal::Text("".into())),
            branch(Literal::Text("ok".into())),
        ],
    }), test_span());

    let mut ev = Evaluator::new();
    ev.eval_node(&node)
    .expect("node evaluation should succeed");

    match ev.get("x") {
        Some(Value::Text(s)) => assert_eq!(s, "ok"),
        other => panic!("expected x = Text(\"ok\"), got {:?}", other),
    }
}

#[test]
fn guard_assigns_void_if_all_branches_false() {
    let node = Node::new(NodeKind::Guard(Guard {
        target: "x".into(),
        branches: vec![
            branch(Literal::Flag(false)),
            branch(Literal::Num(0)),
            branch(Literal::Text("".into())),
        ],
    }), test_span());

    let mut ev = Evaluator::new();
    ev.eval_node(&node)
    .expect("node evaluation should succeed");

    match ev.get("x") {
        Some(Value::Void) => {}
        other => panic!("expected x = Void, got {:?}", other),
    }
}

#[test]
fn guard_single_branch_true() {
    let node = Node::new(NodeKind::Guard(Guard {
        target: "x".into(),
        branches: vec![branch(Literal::Num(5))],
    }), test_span());

    let mut ev = Evaluator::new();
    ev.eval_node(&node)
    .expect("node evaluation should succeed");

    match ev.get("x") {
        Some(Value::Num(n)) => assert_eq!(n, 5),
        other => panic!("expected x = Num(5), got {:?}", other),
    }
}

#[test]
fn guard_single_branch_false_becomes_void() {
    let node = Node::new(NodeKind::Guard(Guard {
        target: "x".into(),
        branches: vec![branch(Literal::Num(0))],
    }), test_span());

    let mut ev = Evaluator::new();
    ev.eval_node(&node)
    .expect("node evaluation should succeed");

    match ev.get("x") {
        Some(Value::Void) => {}
        other => panic!("expected x = Void, got {:?}", other),
    }
}

#[test]
fn function_call_returns_explicit_value() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "identity".to_string(),
                params: vec![Param {
                    name: "value".to_string(),
                    default: None,
                }],
                body: vec![Node::new(NodeKind::Ret(Ret {
                    value: Some(Box::new(Node::new(NodeKind::Ident(
                        "value".to_string(),
                    ), test_span()))),
                }), test_span())],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "identity".to_string(),
                    ), test_span())),
                    args: vec![Node::new(NodeKind::Lit(Literal::Num(7)), test_span())],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::from_literal(&Literal::Num(7)))
    );
}

#[test]
fn function_call_without_return_evaluates_to_void() {
    use crate::compiler::ast::{
        Call, Define, Func, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "noop".to_string(),
                params: vec![],
                body: vec![],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("noop".to_string()), test_span())),
                    args: vec![],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn function_call_binds_multiple_parameters_in_order() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "second".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![Node::new(NodeKind::Ret(Ret {
                    value: Some(Box::new(Node::new(NodeKind::Ident(
                        "b".to_string(),
                    ), test_span()))),
                }), test_span())],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "second".to_string(),
                    ), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(3)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(4)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::from_literal(&Literal::Num(4))),
    );
}

#[test]
fn function_call_uses_default_parameter_value() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "identity".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: Some(Node::new(NodeKind::Lit(Literal::Num(42)), test_span())),
                    },
                ],
                body: vec![Node::new(NodeKind::Ret(Ret {
                    value: Some(Box::new(Node::new(NodeKind::Ident(
                        "value".to_string(),
                    ), test_span()))),
                }), test_span())],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "identity".to_string(),
                    ), test_span())),
                    args: vec![],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::from_literal(&Literal::Num(42))),
    );
}

#[test]
fn function_call_explicit_argument_overrides_default() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "identity".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: Some(Node::new(NodeKind::Lit(Literal::Num(42)), test_span())),
                    },
                ],
                body: vec![Node::new(NodeKind::Ret(Ret {
                    value: Some(Box::new(Node::new(NodeKind::Ident(
                        "value".to_string(),
                    ), test_span()))),
                }), test_span())],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "identity".to_string(),
                    ), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(7)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::from_literal(&Literal::Num(7))),
    );
}

#[test]
fn function_call_rejects_missing_required_argument() {
    use crate::compiler::ast::{
        Call, Func, Node, Param, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "identity".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: None,
                    },
                ],
                body: vec![],
            }), test_span()),

            Node::new(NodeKind::Call(Call {
                callee: Box::new(Node::new(NodeKind::Ident(
                    "identity".to_string(),
                ), test_span())),
                args: vec![],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    let err = evaluator
        .eval_program(&program)
        .expect_err("missing argument should return a diagnostic");

    assert!(
        err.message.contains("missing required argument"),
        "unexpected diagnostic: {:?}",
        err,
    );
}

#[test]
fn function_call_rejects_too_many_arguments() {
    use crate::compiler::ast::{
        Call, Func, Literal, Node, Param, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "identity".to_string(),
                params: vec![Param {
                    name: "value".to_string(),
                    default: None,
                }],
                body: vec![],
            }), test_span()),

            Node::new(NodeKind::Call(Call {
                callee: Box::new(Node::new(NodeKind::Ident(
                    "identity".to_string(),
                ), test_span())),
                args: vec![
                    Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                    Node::new(NodeKind::Lit(Literal::Num(2)), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    let err = evaluator
        .eval_program(&program)
        .expect_err("too many arguments should return a diagnostic");

    assert!(
        err.message.contains("expected at most"),
        "unexpected diagnostic: {:?}",
        err,
    );
}

#[test]
fn function_call_rejects_non_function_callee() {
    use crate::compiler::ast::{
        Call, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Call(Call {
                callee: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(7),
                ), test_span())),
                args: vec![],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    let err = evaluator
        .eval_program(&program)
        .expect_err("non-function call should return a diagnostic");

    assert_eq!(
        err.message,
        "attempted to call a non-function value",
    );
}

#[test]
fn non_function_call_uses_callee_source_span() {
    use crate::compiler::error::Span;
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = "value = 7; value();";

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("non-function call should return a diagnostic");

    assert_eq!(
        err.message,
        "attempted to call a non-function value",
    );

    assert_eq!(
        err.span,
        Span {
            start: 11,
            end: 16,
        },
    );
}

#[test]
fn function_return_stops_remaining_body_execution() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "early_return".to_string(),
                params: vec![],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(1),
                        ), test_span()))),
                    }), test_span()),

                    Node::new(NodeKind::Define(Define {
                        name: "should_not_exist".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(2),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "early_return".to_string(),
                    ), test_span())),
                    args: vec![],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::from_literal(&Literal::Num(1))),
    );

    assert_eq!(
        evaluator.get("should_not_exist"),
        None,
    );
}

#[test]
fn function_return_without_value_evaluates_to_void() {
    use crate::compiler::ast::{
        Call, Define, Func, Node, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "return_void".to_string(),
                params: vec![],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: None,
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "return_void".to_string(),
                    ), test_span())),
                    args: vec![],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn function_return_inside_nested_block_propagates() {
    use crate::compiler::ast::{
        Block, BlockSegment, Call, Define, Func, Literal, Node, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "nested_return".to_string(),
                params: vec![],
                body: vec![
                    Node::new(NodeKind::Block(Block {
                        segments: vec![
                            BlockSegment {
                                nodes: vec![
                                    Node::new(NodeKind::Ret(Ret {
                                        value: Some(Box::new(Node::new(NodeKind::Lit(
                                            Literal::Num(9),
                                        ), test_span()))),
                                    }), test_span()),
                                ],
                            },
                        ],
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "nested_return".to_string(),
                    ), test_span())),
                    args: vec![],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::from_literal(&Literal::Num(9))),
    );
}

#[test]
fn function_scope_is_removed_after_call() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "scoped".to_string(),
                params: vec![
                    Param {
                        name: "input".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Define(Define {
                        name: "local_value".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(9),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Call(Call {
                callee: Box::new(Node::new(NodeKind::Ident(
                    "scoped".to_string(),
                ), test_span())),
                args: vec![
                    Node::new(NodeKind::Lit(Literal::Num(4)), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("input"),
        None,
    );

    assert_eq!(
        evaluator.get("local_value"),
        None,
    );

    assert_eq!(
        evaluator.get("scoped"),
        Some(Value::Func(
            crate::compiler::semantics::value::Func {
                name: "scoped".to_string(),
                params: vec![
                    Param {
                        name: "input".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Define(Define {
                        name: "local_value".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(9),
                        ), test_span())),
                    }), test_span()),
                ],
            },
        )),
    );
}

#[test]
fn nested_function_call_evaluates_as_argument() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let identity = Node::new(NodeKind::Func(Func {
        name: "identity".to_string(),
        params: vec![
            Param {
                name: "value".to_string(),
                default: None,
            },
        ],
        body: vec![
            Node::new(NodeKind::Ret(Ret {
                value: Some(Box::new(Node::new(NodeKind::Ident(
                    "value".to_string(),
                ), test_span()))),
            }), test_span()),
        ],
    }), test_span());

    let inner_call = Node::new(NodeKind::Call(Call {
        callee: Box::new(Node::new(NodeKind::Ident(
            "identity".to_string(),
        ), test_span())),
        args: vec![
            Node::new(NodeKind::Lit(Literal::Num(7)), test_span()),
        ],
    }), test_span());

    let outer_call = Node::new(NodeKind::Call(Call {
        callee: Box::new(Node::new(NodeKind::Ident(
            "identity".to_string(),
        ), test_span())),
        args: vec![inner_call],
    }), test_span());

    let program = Program {
        nodes: vec![
            identity,

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(outer_call),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::from_literal(&Literal::Num(7))),
    );
}

#[test]
fn function_can_return_sum_of_parameters() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "sum".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("sum".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(2)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(3)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(5)),
    );
}

#[test]
fn function_can_return_difference_of_parameters() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "difference".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Sub(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("difference".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(7)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(2)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(evaluator.get("result"), Some(Value::Num(5)));
}

#[test]
fn function_can_return_product_of_parameters() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "product".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Mul(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("product".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(7)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(3)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(21)),
    );
}

#[test]
fn function_can_return_quotient_of_parameters() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "quotient".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Div(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("quotient".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(21)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(3)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(7)),
    );
}

#[test]
fn function_can_return_remainder_of_parameters() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "remainder".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Mod(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("remainder".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(22)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(5)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(2)),
    );
}

#[test]
fn division_by_zero_returns_diagnostic() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "quotient".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Div(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("quotient".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(7)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(0)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    let err = evaluator
        .eval_program(&program)
        .expect_err("division by zero should return a diagnostic");

    assert_eq!(err.message, "division by zero");
}

#[test]
fn modulo_by_zero_returns_diagnostic() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "remainder".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Mod(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("remainder".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(7)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(0)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
        let err = evaluator
        .eval_program(&program)
        .expect_err("modulo by zero should return a diagnostic");

    assert_eq!(err.message, "modulo by zero");
}

#[test]
fn function_can_return_equality_comparison() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "equals".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Eq(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("equals".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(5)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(5)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn function_can_return_inequality_comparison() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "not_equals".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Ne(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("not_equals".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(5)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(6)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn function_can_return_less_than_comparison() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "less_than".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Lt(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("less_than".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(4)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(7)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn function_can_return_less_than_or_equal_comparison() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "less_than_or_equal".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Le(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("less_than_or_equal".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(7)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(7)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn function_can_return_greater_than_comparison() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "greater_than".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Gt(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("greater_than".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(9)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(4)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn function_can_return_greater_than_or_equal_comparison() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "greater_than_or_equal".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Ge(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("greater_than_or_equal".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(9)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(9)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn function_can_return_logical_and() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "logical_and".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::And(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("logical_and".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Flag(true)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Flag(true)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn function_can_return_logical_or() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "logical_or".to_string(),
                params: vec![
                    Param {
                        name: "a".to_string(),
                        default: None,
                    },
                    Param {
                        name: "b".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Or(
                            Box::new(Node::new(NodeKind::Ident("a".to_string()), test_span())),
                            Box::new(Node::new(NodeKind::Ident("b".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("logical_or".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Flag(false)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Flag(true)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn function_can_return_logical_not() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "logical_not".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Not(
                            Box::new(Node::new(NodeKind::Ident("value".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("logical_not".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Flag(false)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn function_can_return_numeric_negation() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "negate".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Neg(
                            Box::new(Node::new(NodeKind::Ident("value".to_string()), test_span())),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident("negate".to_string()), test_span())),
                    args: vec![
                        Node::new(NodeKind::Lit(Literal::Num(42)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(-42)),
    );
}

#[test]
fn get_on_void_returns_void() {
    use crate::compiler::ast::{Define, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Lit(crate::compiler::ast::Literal::Void), test_span())),
                    Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn has_on_void_returns_false() {
    use crate::compiler::ast::{Define, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Lit(crate::compiler::ast::Literal::Void), test_span())),
                    Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(false)),
    );
}

#[test]
fn get_on_flag_returns_void() {
    use crate::compiler::ast::{Define, Literal, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![Node::new(NodeKind::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::new(NodeKind::Get(
                Box::new(Node::new(NodeKind::Lit(Literal::Flag(true)), test_span())),
                Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
            ), test_span())),
        }), test_span())],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn has_on_flag_returns_false() {
    use crate::compiler::ast::{Define, Literal, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![Node::new(NodeKind::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::new(NodeKind::Has(
                Box::new(Node::new(NodeKind::Lit(Literal::Flag(true)), test_span())),
                Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
            ), test_span())),
        }), test_span())],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(false)),
    );
}

#[test]
fn get_on_num_returns_void() {
    use crate::compiler::ast::{Define, Literal, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![Node::new(NodeKind::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::new(NodeKind::Get(
                Box::new(Node::new(NodeKind::Lit(Literal::Num(123)), test_span())),
                Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
            ), test_span())),
        }), test_span())],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn has_on_num_returns_false() {
    use crate::compiler::ast::{Define, Literal, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![Node::new(NodeKind::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::new(NodeKind::Has(
                Box::new(Node::new(NodeKind::Lit(Literal::Num(123)), test_span())),
                Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
            ), test_span())),
        }), test_span())],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(false)),
    );
}

#[test]
fn get_on_text_returns_void() {
    use crate::compiler::ast::{Define, Literal, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![Node::new(NodeKind::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::new(NodeKind::Get(
                Box::new(Node::new(NodeKind::Lit(Literal::Text("hello".to_string())), test_span())),
                Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
            ), test_span())),
        }), test_span())],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn has_on_text_returns_false() {
    use crate::compiler::ast::{Define, Literal, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![Node::new(NodeKind::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::new(NodeKind::Has(
                Box::new(Node::new(NodeKind::Lit(Literal::Text("hello".to_string())), test_span())),
                Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
            ), test_span())),
        }), test_span())],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(false)),
    );
}

#[test]
fn get_on_dec_returns_void() {
    use crate::compiler::ast::{Define, Literal, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![Node::new(NodeKind::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::new(NodeKind::Get(
                Box::new(Node::new(NodeKind::Lit(Literal::Dec("12.34".to_string())), test_span())),
                Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
            ), test_span())),
        }), test_span())],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn has_on_dec_returns_false() {
    use crate::compiler::ast::{Define, Literal, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![Node::new(NodeKind::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::new(NodeKind::Has(
                Box::new(Node::new(NodeKind::Lit(Literal::Dec("12.34".to_string())), test_span())),
                Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
            ), test_span())),
        }), test_span())],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(false)),
    );
}

#[test]
fn get_on_void_identifier_returns_void() {
    use crate::compiler::ast::{Define, Literal, Node, Program};
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "x".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Void), test_span())),
            }), test_span()),
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("x".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Ident("anything".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn evaluates_box_literal() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(2)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(3)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("items"),
        Some(Value::Box(vec![
            Value::Num(1),
            Value::Num(2),
            Value::Num(3),
        ])),
    );
}

#[test]
fn evaluates_bag_literal() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::new(NodeKind::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )), test_span()),
                        },
                        BagEntry {
                            name: "level".to_string(),
                            value: Node::new(NodeKind::Lit(Literal::Num(42)), test_span()),
                        },
                        BagEntry {
                            name: "active".to_string(),
                            value: Node::new(NodeKind::Lit(Literal::Flag(true)), test_span()),
                        },
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    let Some(Value::Bag(entries)) = evaluator.get("player") else {
        panic!("expected Bag value");
    };

    assert_eq!(
        entries.get("name"),
        Some(&Value::Text("Rusty".to_string())),
    );

    assert_eq!(
        entries.get("level"),
        Some(&Value::Num(42)),
    );

    assert_eq!(
        entries.get("active"),
        Some(&Value::Flag(true)),
    );
}

#[test]
fn get_returns_named_bag_value() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::new(NodeKind::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )), test_span()),
                        },
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("player".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Ident("name".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Text("Rusty".to_string())),
    );
}

#[test]
fn get_returns_void_for_missing_bag_name() {
    use crate::compiler::ast::{
        BagLiteral, Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("player".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Ident("missing".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn has_returns_true_for_existing_bag_name() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::new(NodeKind::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )), test_span()),
                        },
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("player".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Ident("name".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn has_returns_false_for_missing_bag_name() {
    use crate::compiler::ast::{
        BagLiteral, Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("player".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Ident("missing".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(false)),
    );
}

#[test]
fn get_returns_box_value_at_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Text("first".to_string())), test_span()),
                        Node::new(NodeKind::Lit(Literal::Text("second".to_string())), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Text("second".to_string())),
    );
}

#[test]
fn get_returns_void_for_out_of_range_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Num(10)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Num(4)), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );
}

#[test]
fn has_returns_true_for_existing_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Void), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Num(0)), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn has_returns_false_for_missing_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Num(0)), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(false)),
    );
}

#[test]
fn get_traverses_from_box_into_nested_bag() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Bag(BagLiteral {
                            entries: vec![
                                BagEntry {
                                    name: "name".to_string(),
                                    value: Node::new(NodeKind::Lit(Literal::Text(
                                        "Rusty".to_string(),
                                    )), test_span()),
                                },
                            ],
                        }), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Get(
                        Box::new(Node::new(NodeKind::Ident("data".to_string()), test_span())),
                        Box::new(Node::new(NodeKind::Index(Box::new(
                            Node::new(NodeKind::Lit(Literal::Num(0)), test_span()),
                        )), test_span())),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Ident("name".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Text("Rusty".to_string())),
    );
}


#[test]
fn get_traverses_nested_boxes_by_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "matrix".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Box(BoxLiteral {
                            values: vec![
                                Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                                Node::new(NodeKind::Lit(Literal::Num(2)), test_span()),
                            ],
                        }), test_span()),
                        Node::new(NodeKind::Box(BoxLiteral {
                            values: vec![
                                Node::new(NodeKind::Lit(Literal::Num(3)), test_span()),
                                Node::new(NodeKind::Lit(Literal::Num(4)), test_span()),
                            ],
                        }), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Get(
                        Box::new(Node::new(NodeKind::Ident("matrix".to_string()), test_span())),
                        Box::new(Node::new(NodeKind::Index(Box::new(
                            Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                        )), test_span())),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Num(0)), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(3)),
    );
}

#[test]
fn get_traverses_from_bag_into_nested_box() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "items".to_string(),
                            value: Node::new(NodeKind::Box(BoxLiteral {
                                values: vec![
                                    Node::new(NodeKind::Lit(Literal::Text(
                                        "first".to_string(),
                                    )), test_span()),
                                    Node::new(NodeKind::Lit(Literal::Text(
                                        "second".to_string(),
                                    )), test_span()),
                                ],
                            }), test_span()),
                        },
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Get(
                        Box::new(Node::new(NodeKind::Ident("data".to_string()), test_span())),
                        Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Text("second".to_string())),
    );
}

#[test]
fn has_checks_named_entry_after_box_traversal() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Bag(BagLiteral {
                            entries: vec![
                                BagEntry {
                                    name: "name".to_string(),
                                    value: Node::new(NodeKind::Lit(Literal::Text(
                                        "Rusty".to_string(),
                                    )), test_span()),
                                },
                            ],
                        }), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Get(
                        Box::new(Node::new(NodeKind::Ident("data".to_string()), test_span())),
                        Box::new(Node::new(NodeKind::Index(Box::new(
                            Node::new(NodeKind::Lit(Literal::Num(0)), test_span()),
                        )), test_span())),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Ident("name".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn has_checks_box_index_after_bag_traversal() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "items".to_string(),
                            value: Node::new(NodeKind::Box(BoxLiteral {
                                values: vec![
                                    Node::new(NodeKind::Lit(Literal::Num(10)), test_span()),
                                    Node::new(NodeKind::Lit(Literal::Num(20)), test_span()),
                                ],
                            }), test_span()),
                        },
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Get(
                        Box::new(Node::new(NodeKind::Ident("data".to_string()), test_span())),
                        Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn get_rejects_named_selector_on_box() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Ident("name".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("named Box traversal should return a diagnostic");

    assert_eq!(
        err.message,
        "Box traversal requires an indexed selector",
    );
}

#[test]
fn get_rejects_indexed_selector_on_bag() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::new(NodeKind::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )), test_span()),
                        },
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("player".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Num(0)), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("indexed Bag traversal should return a diagnostic");

    assert_eq!(
        err.message,
        "Bag traversal requires a named selector",
    );
}

#[test]
fn has_rejects_named_selector_on_box() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Ident("name".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("named Box existence check should return a diagnostic");

    assert_eq!(
        err.message,
        "Box traversal requires an indexed selector",
    );
}

#[test]
fn has_evaluates_computed_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Num(10)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(20)), test_span()),
                        Node::new(NodeKind::Lit(Literal::Num(30)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Lit(Literal::Num(1)), test_span())),
                            Box::new(Node::new(NodeKind::Lit(Literal::Num(1)), test_span())),
                        ), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn has_rejects_indexed_selector_on_bag() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::new(NodeKind::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )), test_span()),
                        },
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("player".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Num(0)), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("indexed Bag existence check should return a diagnostic");

    assert_eq!(
        err.message,
        "Bag traversal requires a named selector",
    );
}

#[test]
fn nested_traversal_propagates_selector_mismatch_diagnostic() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "items".to_string(),
                            value: Node::new(NodeKind::Box(BoxLiteral {
                                values: vec![
                                    Node::new(NodeKind::Lit(Literal::Num(10)), test_span()),
                                ],
                            }), test_span()),
                        },
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Get(
                        Box::new(Node::new(NodeKind::Ident("data".to_string()), test_span())),
                        Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Ident("name".to_string()), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("nested selector mismatch should return a diagnostic");

    assert_eq!(
        err.message,
        "Box traversal requires an indexed selector",
    );
}

#[test]
fn get_rejects_negative_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Text("first".to_string())), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Neg(Box::new(
                            Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                        )), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("negative Box index should return a diagnostic");

    assert_eq!(
        err.message,
        "Box index cannot be negative",
    );
}

#[test]
fn get_rejects_non_numeric_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Text("first".to_string())), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Text("zero".to_string())), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("non-numeric Box index should return a diagnostic");

    assert_eq!(
        err.message,
        "Box index must evaluate to a number",
    );
}

#[test]
fn has_rejects_negative_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Num(10)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Neg(Box::new(
                            Node::new(NodeKind::Lit(Literal::Num(1)), test_span()),
                        )), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("negative Box index should return a diagnostic");

    assert_eq!(
        err.message,
        "Box index cannot be negative",
    );
}

#[test]
fn has_rejects_non_numeric_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Num(10)), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Lit(Literal::Text(
                            "zero".to_string(),
                        )), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("non-numeric Box index should return a diagnostic");

    assert_eq!(
        err.message,
        "Box index must evaluate to a number",
    );
}

#[test]
fn get_evaluates_computed_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Text("zero".to_string())), test_span()),
                        Node::new(NodeKind::Lit(Literal::Text("one".to_string())), test_span()),
                        Node::new(NodeKind::Lit(Literal::Text("two".to_string())), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Lit(Literal::Num(1)), test_span())),
                            Box::new(Node::new(NodeKind::Lit(Literal::Num(1)), test_span())),
                        ), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Text("two".to_string())),
    );
}

#[test]
fn get_evaluates_identifier_box_index() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "index".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(1)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Text("zero".to_string())), test_span()),
                        Node::new(NodeKind::Lit(Literal::Text("one".to_string())), test_span()),
                    ],
                }), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Ident("index".to_string()), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();
    evaluator
    .eval_program(&program)
    .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Text("one".to_string())),
    );
}

#[test]
fn undeclared_identifier_produces_diagnostic() {
    use crate::compiler::ast::{
        Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Ident(
                    "missing_value".to_string(),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared identifier should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn indexed_get_propagates_undeclared_index_identifier_diagnostic() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Text("zero".to_string())), test_span()),
                        Node::new(NodeKind::Lit(Literal::Text("one".to_string())), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Ident("missing_index".to_string()), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared index identifier should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_index`",
    );
}

#[test]
fn indexed_has_propagates_undeclared_index_identifier_diagnostic() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Lit(Literal::Text("zero".to_string())), test_span()),
                        Node::new(NodeKind::Lit(Literal::Text("one".to_string())), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident("items".to_string()), test_span())),
                    Box::new(Node::new(NodeKind::Index(Box::new(
                        Node::new(NodeKind::Ident("missing_index".to_string()), test_span()),
                    )), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared index identifier should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_index`",
    );
}

#[test]
fn get_propagates_undeclared_target_identifier_diagnostic() {
    use crate::compiler::ast::{
        Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Get(
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_container".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Ident(
                        "name".to_string(),
                    ), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared traversal target should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_container`",
    );
}

#[test]
fn has_propagates_undeclared_target_identifier_diagnostic() {
    use crate::compiler::ast::{
        Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Has(
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_container".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Ident(
                        "name".to_string(),
                    ), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared traversal target should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_container`",
    );
}

#[test]
fn guard_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        Guard, GuardBranch, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Guard(Guard {
                target: "result".to_string(),
                branches: vec![
                    GuardBranch {
                        expr: Node::new(NodeKind::Ident(
                            "missing_value".to_string(),
                        ), test_span()),
                    },
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared Guard branch should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn function_return_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        Call, Define, Func, Node, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "read_missing".to_string(),
                params: vec![],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Ident(
                            "missing_value".to_string(),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "read_missing".to_string(),
                    ), test_span())),
                    args: vec![],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared return value should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn function_argument_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        Call, Define, Func, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "identity".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Ident(
                            "value".to_string(),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "identity".to_string(),
                    ), test_span())),
                    args: vec![
                        Node::new(NodeKind::Ident(
                            "missing_argument".to_string(),
                        ), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared function argument should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_argument`",
    );
}

#[test]
fn default_parameter_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        Call, Define, Func, Node, Param, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "read_default".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: Some(Node::new(NodeKind::Ident(
                            "missing_default".to_string(),
                        ), test_span())),
                    },
                ],
                body: vec![
                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Ident(
                            "value".to_string(),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "read_default".to_string(),
                    ), test_span())),
                    args: vec![],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared default parameter value should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_default`",
    );
}

#[test]
fn box_literal_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        BoxLiteral, Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Box(BoxLiteral {
                    values: vec![
                        Node::new(NodeKind::Ident(
                            "missing_value".to_string(),
                        ), test_span()),
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared Box value should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn bag_literal_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "value".to_string(),
                            value: Node::new(NodeKind::Ident(
                                "missing_value".to_string(),
                            ), test_span()),
                        },
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared Bag value should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn function_call_propagates_undeclared_callee_diagnostic() {
    use crate::compiler::ast::{
        Call, Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "missing_function".to_string(),
                    ), test_span())),
                    args: vec![],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared function callee should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_function`",
    );
}

#[test]
fn arithmetic_expression_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Add(
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_value".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(Literal::Num(1)), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared arithmetic operand should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn equality_comparison_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Eq(
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_value".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(Literal::Num(1)), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared comparison operand should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn logical_and_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::And(
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_value".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(Literal::Flag(true)), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared logical operand should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn logical_not_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Not(
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_value".to_string(),
                    ), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared logical operand should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn numeric_negation_propagates_undeclared_identifier_diagnostic() {
    use crate::compiler::ast::{
        Define, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Neg(
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_value".to_string(),
                    ), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "undeclared numeric operand should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn logical_and_short_circuits_false_left_operand() {
    use crate::compiler::ast::{
        Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::And(
                    Box::new(Node::new(NodeKind::Lit(Literal::Flag(false)), test_span())),
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_value".to_string(),
                    ), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect(
            "false AND should not evaluate the right-hand operand",
        );

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(false)),
    );
}

#[test]
fn logical_or_short_circuits_true_left_operand() {
    use crate::compiler::ast::{
        Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Or(
                    Box::new(Node::new(NodeKind::Lit(Literal::Flag(true)), test_span())),
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_value".to_string(),
                    ), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect(
            "true OR should not evaluate the right-hand operand",
        );

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Flag(true)),
    );
}

#[test]
fn logical_or_evaluates_right_operand_when_left_is_false() {
    use crate::compiler::ast::{
        Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Or(
                    Box::new(Node::new(NodeKind::Lit(Literal::Flag(false)), test_span())),
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_value".to_string(),
                    ), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "false OR should evaluate the right-hand operand",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn logical_and_evaluates_right_operand_when_left_is_true() {
    use crate::compiler::ast::{
        Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::And(
                    Box::new(Node::new(NodeKind::Lit(Literal::Flag(true)), test_span())),
                    Box::new(Node::new(NodeKind::Ident(
                        "missing_value".to_string(),
                    ), test_span())),
                ), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "true AND should evaluate the right-hand operand",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_value`",
    );
}

#[test]
fn copy_from_undeclared_identifier_produces_diagnostic() {
    use crate::compiler::ast::{
        Copy, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Copy(Copy {
                name: "result".to_string(),
                target: "missing_source".to_string(),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "Copy from an undeclared identifier should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_source`",
    );
}

#[test]
fn copy_from_undeclared_identifier_uses_statement_source_span() {
    use crate::compiler::error::Span;
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = "result := missing_source;";

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "Copy from an undeclared identifier should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_source`",
    );

    assert_eq!(
        err.span,
        Span {
            start: 0,
            end: 25,
        },
    );
}

#[test]
fn bind_from_undeclared_identifier_produces_diagnostic() {
    use crate::compiler::ast::{
        Bind, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Bind(Bind {
                name: "result".to_string(),
                target: "missing_source".to_string(),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "Bind from an undeclared identifier should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_source`",
    );
}

#[test]
fn bind_from_undeclared_identifier_uses_statement_source_span() {
    use crate::compiler::error::Span;
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = "result :> missing_source;";

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "Bind from an undeclared identifier should produce a diagnostic",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `missing_source`",
    );

    assert_eq!(
        err.span,
        Span {
            start: 0,
            end: 25,
        },
    );
}

#[test]
fn bind_preserves_shared_identity_after_source_mutation() {
    use crate::compiler::ast::{
        Bind, Define, Literal, Mutate, Node, NodeKind, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(
                NodeKind::Define(Define {
                    name: "source".to_string(),
                    value: Box::new(Node::new(
                        NodeKind::Lit(Literal::Num(10)),
                        test_span(),
                    )),
                }),
                test_span(),
            ),
            Node::new(
                NodeKind::Bind(Bind {
                    name: "alias".to_string(),
                    target: "source".to_string(),
                }),
                test_span(),
            ),
            Node::new(
                NodeKind::Mutate(Mutate {
                    name: "source".to_string(),
                    value: Box::new(Node::new(
                        NodeKind::Lit(Literal::Num(20)),
                        test_span(),
                    )),
                }),
                test_span(),
            ),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("alias"),
        Some(Value::Num(20)),
    );
}

#[test]
fn bind_preserves_shared_identity_after_alias_mutation() {
    use crate::compiler::ast::{
        Bind, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Bind(Bind {
                name: "alias".to_string(),
                target: "source".to_string(),
            }), test_span()),

           Node::new(NodeKind::Mutate(Mutate {
                name: "alias".to_string(),
                value: Box::new(Node::new(
                    NodeKind::Lit(Literal::Num(20)),
                    test_span(),
                )),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(20)),
    );
}

#[test]
fn copy_remains_independent_after_source_mutation() {
    use crate::compiler::ast::{
        Copy, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Copy(Copy {
                name: "snapshot".to_string(),
                target: "source".to_string(),
            }), test_span()),
            Node::new(NodeKind::Mutate(Mutate {
                name: "source".to_string(),
                value: Box::new(Node::new(
                    NodeKind::Lit(Literal::Num(20)),
                    test_span(),
                )),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("snapshot"),
        Some(Value::Num(10)),
    );

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(20)),
    );
}

#[test]
fn copy_mutation_does_not_affect_source() {
    use crate::compiler::ast::{
        Copy, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Copy(Copy {
                name: "snapshot".to_string(),
                target: "source".to_string(),
            }), test_span()),

            Node::new(NodeKind::Mutate(Mutate {
                name: "snapshot".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(20)), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10)),
    );

    assert_eq!(
        evaluator.get("snapshot"),
        Some(Value::Num(20)),
    );
}

#[test]
fn bind_preserves_transitive_shared_identity_after_mutation() {
    use crate::compiler::ast::{
        Bind, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "a".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Bind(Bind {
                name: "b".to_string(),
                target: "a".to_string(),
            }), test_span()),

            Node::new(NodeKind::Bind(Bind {
                name: "c".to_string(),
                target: "b".to_string(),
            }), test_span()),

            Node::new(NodeKind::Mutate(Mutate {
                name: "c".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(30)), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("program evaluation should succeed");

    assert_eq!(evaluator.get("a"), Some(Value::Num(30)));
    assert_eq!(evaluator.get("b"), Some(Value::Num(30)));
    assert_eq!(evaluator.get("c"), Some(Value::Num(30)));
}

#[test]
fn copy_from_bound_alias_remains_independent_after_source_mutation() {
    use crate::compiler::ast::{
        Bind, Copy, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Bind(Bind {
                name: "alias".to_string(),
                target: "source".to_string(),
            }), test_span()),

            Node::new(NodeKind::Copy(Copy {
                name: "snapshot".to_string(),
                target: "alias".to_string(),
            }), test_span()),

            Node::new(NodeKind::Mutate(Mutate {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(20)), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(20)),
    );

    assert_eq!(
        evaluator.get("alias"),
        Some(Value::Num(20)),
    );

    assert_eq!(
        evaluator.get("snapshot"),
        Some(Value::Num(10)),
    );
}

#[test]
fn multiple_bind_aliases_share_identity() {
    use crate::compiler::ast::{
        Bind, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Bind(Bind {
                name: "alias_one".to_string(),
                target: "source".to_string(),
            }), test_span()),

            Node::new(NodeKind::Bind(Bind {
                name: "alias_two".to_string(),
                target: "source".to_string(),
            }), test_span()),

            Node::new(NodeKind::Mutate(Mutate {
                name: "alias_one".to_string(),
                value: Box::new(Node::new(
                    NodeKind::Lit(Literal::Num(40)),
                    test_span(),
                )),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(40)),
    );

    assert_eq!(
        evaluator.get("alias_one"),
        Some(Value::Num(40)),
    );

    assert_eq!(
        evaluator.get("alias_two"),
        Some(Value::Num(40)),
    );
}

#[test]
fn inner_scope_define_rejects_visible_bound_source() {
    use crate::compiler::ast::{
        Bind, Block, BlockSegment, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Bind(Bind {
                name: "alias".to_string(),
                target: "source".to_string(),
            }), test_span()),

            Node::new(NodeKind::Block(Block {
                segments: vec![
                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Define(Define {
                                name: "source".to_string(),
                                value: Box::new(Node::new(NodeKind::Lit(
                                    Literal::Num(20),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("inner scope should not redefine a visible binding");

    assert!(
        error.message.contains("identifier `source` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn inner_scope_define_rejects_visible_bound_alias() {
    use crate::compiler::ast::{
        Bind, Block, BlockSegment, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Bind(Bind {
                name: "alias".to_string(),
                target: "source".to_string(),
            }), test_span()),

            Node::new(NodeKind::Block(Block {
                segments: vec![
                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Define(Define {
                                name: "alias".to_string(),
                                value: Box::new(Node::new(NodeKind::Lit(
                                    Literal::Num(20),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
    .eval_program(&program)
    .expect_err("inner scope should not redefine a visible binding");

    assert!(
        error.message.contains("identifier `alias` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn inner_scope_bound_alias_mutation_updates_outer_identity() {
    use crate::compiler::ast::{
        Bind, Block, BlockSegment, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Block(Block {
                segments: vec![
                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Bind(Bind {
                                name: "alias".to_string(),
                                target: "source".to_string(),
                            }), test_span()),

                            Node::new(NodeKind::Mutate(Mutate {
                                name: "alias".to_string(),
                                value: Box::new(Node::new(NodeKind::Lit(
                                    Literal::Num(20),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(20)),
    );

    assert_eq!(
        evaluator.get("alias"),
        None,
    );
}

#[test]
fn inner_scope_copy_mutation_does_not_mutate_outer_source() {
    use crate::compiler::ast::{
        Block, BlockSegment, Copy, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Block(Block {
                segments: vec![
                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Copy(Copy {
                                name: "snapshot".to_string(),
                                target: "source".to_string(),
                            }), test_span()),

                            Node::new(NodeKind::Mutate(Mutate {
                                name: "snapshot".to_string(),
                                value: Box::new(Node::new(NodeKind::Lit(
                                    Literal::Num(20),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("program evaluation should succeed");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10)),
    );

    assert_eq!(
        evaluator.get("snapshot"),
        None,
    );
}

#[test]
fn local_binding_is_unavailable_in_later_block_segment() {
    use crate::compiler::ast::{
        Block, BlockSegment, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Block(Block {
                segments: vec![
                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Local(Box::new(
                                Node::new(NodeKind::Define(Define {
                                    name: "temporary".to_string(),
                                    value: Box::new(Node::new(NodeKind::Lit(
                                        Literal::Num(10),
                                    ), test_span())),
                                }), test_span()),
                            )), test_span()),
                        ],
                    },

                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Define(Define {
                                name: "result".to_string(),
                                value: Box::new(Node::new(NodeKind::Ident(
                                    "temporary".to_string(),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err(
            "local binding should not survive into a later block segment",
        );

    assert_eq!(
        err.message,
        "undeclared identifier `temporary`",
    );
}

#[test]
fn local_binding_is_available_within_its_block_segment() {
    use crate::compiler::ast::{
        Block, BlockSegment, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Block(Block {
                segments: vec![
                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Local(Box::new(
                                Node::new(NodeKind::Define(Define {
                                    name: "temporary".to_string(),
                                    value: Box::new(Node::new(NodeKind::Lit(
                                        Literal::Num(10),
                                    ), test_span())),
                                }), test_span()),
                            )), test_span()),

                            Node::new(NodeKind::Define(Define {
                                name: "result".to_string(),
                                value: Box::new(Node::new(NodeKind::Ident(
                                    "temporary".to_string(),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("local binding should be available within its segment");

    assert_eq!(
        evaluator.get("result"),
        None,
    );

    assert_eq!(
        evaluator.get("temporary"),
        None,
    );

    // The block-local result existed during evaluation but both names
    // correctly disappear when the block scope ends.
    assert_eq!(
        evaluator.get("temporary"),
        None,
    );

    let _ = Value::Num(10);
}

#[test]
fn local_bind_mutation_updates_source_before_segment_cleanup() {
    use crate::compiler::ast::{
        Bind, Block, BlockSegment, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Block(Block {
                segments: vec![
                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Local(Box::new(
                                Node::new(NodeKind::Bind(Bind {
                                    name: "alias".to_string(),
                                    target: "source".to_string(),
                                }), test_span()),
                            )), test_span()),

                            Node::new(NodeKind::Mutate(Mutate {
                                name: "alias".to_string(),
                                value: Box::new(Node::new(NodeKind::Lit(
                                    Literal::Num(20),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },

                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Define(Define {
                                name: "result".to_string(),
                                value: Box::new(Node::new(NodeKind::Ident(
                                    "source".to_string(),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("local bind should update its shared source slot");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(20)),
    );

    assert_eq!(
        evaluator.get("alias"),
        None,
    );
}

#[test]
fn local_copy_mutation_remains_independent_and_is_removed_after_segment() {
    use crate::compiler::ast::{
        Block, BlockSegment, Copy, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(Literal::Num(10)), test_span())),
            }), test_span()),

            Node::new(NodeKind::Block(Block {
                segments: vec![
                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Local(Box::new(
                                Node::new(NodeKind::Copy(Copy {
                                    name: "snapshot".to_string(),
                                    target: "source".to_string(),
                                }), test_span()),
                            )), test_span()),

                            Node::new(NodeKind::Mutate(Mutate {
                                name: "snapshot".to_string(),
                                value: Box::new(Node::new(NodeKind::Lit(
                                    Literal::Num(20),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },

                    BlockSegment {
                        nodes: vec![
                            Node::new(NodeKind::Define(Define {
                                name: "result".to_string(),
                                value: Box::new(Node::new(NodeKind::Ident(
                                    "source".to_string(),
                                ), test_span())),
                            }), test_span()),
                        ],
                    },
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("local copy should remain independent");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10)),
    );

    assert_eq!(
        evaluator.get("snapshot"),
        None,
    );
}

#[test]
fn local_empty_definition_is_void_within_segment_and_removed_afterward() {
    use crate::compiler::ast::{
        Block, BlockSegment, Define, DefineEmpty, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Block(Block {
                    segments: vec![
                        BlockSegment {
                            nodes: vec![
                                Node::new(NodeKind::Local(Box::new(
                                    Node::new(NodeKind::DefineEmpty(DefineEmpty {
                                        name: "temporary".to_string(),
                                    }), test_span()),
                                )), test_span()),

                                Node::new(NodeKind::Ident(
                                    "temporary".to_string(),
                                ), test_span()),
                            ],
                        },
                    ],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("local empty definition should evaluate as explicit void");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Void),
    );

    assert_eq!(
        evaluator.get("temporary"),
        None,
    );
}

#[test]
fn ordinary_statement_block_binding_remains_available_across_segments() {
    use crate::compiler::ast::{
        Bind, Block, BlockSegment, Define, Literal, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(
                NodeKind::Define(Define {
                    name: "source".to_string(),
                    value: Box::new(Node::new(
                        NodeKind::Lit(Literal::Num(10)),
                        test_span(),
                    )),
                }),
                test_span(),
            ),

            Node::new(
                NodeKind::Block(Block {
                    segments: vec![
                        BlockSegment {
                            nodes: vec![
                                Node::new(
                                    NodeKind::Bind(Bind {
                                        name: "alias".to_string(),
                                        target: "source".to_string(),
                                    }),
                                    test_span(),
                                ),
                            ],
                        },

                        BlockSegment {
                            nodes: vec![
                                Node::new(
                                    NodeKind::Mutate(Mutate {
                                        name: "alias".to_string(),
                                        value: Box::new(Node::new(
                                            NodeKind::Lit(Literal::Num(20)),
                                            test_span(),
                                        )),
                                    }),
                                    test_span(),
                                ),
                            ],
                        },
                    ],
                }),
                test_span(),
            ),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("ordinary statement-block binding should survive later segments");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(20)),
    );

    assert_eq!(
        evaluator.get("alias"),
        None,
    );
}

#[test]
fn loop_updates_outer_binding_until_condition_is_false() {
    use crate::compiler::ast::{
        Define, Literal, Loop, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "count".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![
                    Node::new(NodeKind::Define(Define {
                        name: "limit".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(3),
                        ), test_span())),
                    }), test_span()),
                ],

                condition: Box::new(Node::new(NodeKind::Lt(
                    Box::new(Node::new(NodeKind::Ident(
                        "count".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Ident(
                        "limit".to_string(),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Mutate(Mutate {
                        name: "count".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("loop evaluation should succeed");

    assert_eq!(
        evaluator.get("count"),
        Some(Value::Num(3)),
    );

    assert_eq!(
        evaluator.get("limit"),
        None,
    );
}

#[test]
fn loop_binding_persists_across_iterations() {
    use crate::compiler::ast::{
        Define, Literal, Loop, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "count".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![
                    Node::new(NodeKind::Define(Define {
                        name: "carried".to_string(),
                        value: Box::new(Node::new(
                            NodeKind::Lit(Literal::Num(0)),
                            test_span(),
                        )),
                    }), test_span()),
                ],

                condition: Box::new(Node::new(NodeKind::Or(
                    Box::new(Node::new(NodeKind::Eq(
                        Box::new(Node::new(NodeKind::Ident(
                            "count".to_string(),
                        ), test_span())),
                        Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(0),
                        ), test_span())),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lt(
                        Box::new(Node::new(NodeKind::Ident(
                            "carried".to_string(),
                        ), test_span())),
                        Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(3),
                        ), test_span())),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Mutate(Mutate {
                        name: "carried".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),

                    Node::new(NodeKind::Mutate(Mutate {
                        name: "count".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("loop evaluation should succeed");

    assert_eq!(
        evaluator.get("count"),
        Some(Value::Num(3)),
    );

    assert_eq!(
        evaluator.get("carried"),
        None,
    );
}

#[test]
fn nested_loops_use_independent_persistent_scopes() {
    use crate::compiler::ast::{
        Define, Literal, Loop, Mutate, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(
                NodeKind::Define(Define {
                    name: "total".to_string(),
                    value: Box::new(Node::new(
                        NodeKind::Lit(Literal::Num(0)),
                        test_span(),
                    )),
                }),
                test_span(),
            ),

            Node::new(
                NodeKind::Loop(Loop {
                    setup: vec![
                        Node::new(
                            NodeKind::Define(Define {
                                name: "outer_index".to_string(),
                                value: Box::new(Node::new(
                                    NodeKind::Lit(Literal::Num(0)),
                                    test_span(),
                                )),
                            }),
                            test_span(),
                        ),
                    ],

                    condition: Box::new(Node::new(
                        NodeKind::Lt(
                            Box::new(Node::new(
                                NodeKind::Ident(
                                    "outer_index".to_string(),
                                ),
                                test_span(),
                            )),
                            Box::new(Node::new(
                                NodeKind::Lit(Literal::Num(2)),
                                test_span(),
                            )),
                        ),
                        test_span(),
                    )),

                    process: vec![
                        Node::new(
                            NodeKind::Loop(Loop {
                                setup: vec![
                                    Node::new(
                                        NodeKind::Define(Define {
                                            name: "inner_index".to_string(),
                                            value: Box::new(Node::new(
                                                NodeKind::Lit(
                                                    Literal::Num(0),
                                                ),
                                                test_span(),
                                            )),
                                        }),
                                        test_span(),
                                    ),
                                ],

                                condition: Box::new(Node::new(
                                    NodeKind::Lt(
                                        Box::new(Node::new(
                                            NodeKind::Ident(
                                                "inner_index".to_string(),
                                            ),
                                            test_span(),
                                        )),
                                        Box::new(Node::new(
                                            NodeKind::Lit(
                                                Literal::Num(2),
                                            ),
                                            test_span(),
                                        )),
                                    ),
                                    test_span(),
                                )),

                                process: vec![
                                    Node::new(
                                        NodeKind::Mutate(Mutate {
                                            name: "total".to_string(),
                                            value: Box::new(Node::new(
                                                NodeKind::Add(
                                                    Box::new(Node::new(
                                                        NodeKind::Ident(
                                                            "total".to_string(),
                                                        ),
                                                        test_span(),
                                                    )),
                                                    Box::new(Node::new(
                                                        NodeKind::Lit(
                                                            Literal::Num(1),
                                                        ),
                                                        test_span(),
                                                    )),
                                                ),
                                                test_span(),
                                            )),
                                        }),
                                        test_span(),
                                    ),

                                    Node::new(
                                        NodeKind::Mutate(Mutate {
                                            name: "inner_index".to_string(),
                                            value: Box::new(Node::new(
                                                NodeKind::Add(
                                                    Box::new(Node::new(
                                                        NodeKind::Ident(
                                                            "inner_index"
                                                                .to_string(),
                                                        ),
                                                        test_span(),
                                                    )),
                                                    Box::new(Node::new(
                                                        NodeKind::Lit(
                                                            Literal::Num(1),
                                                        ),
                                                        test_span(),
                                                    )),
                                                ),
                                                test_span(),
                                            )),
                                        }),
                                        test_span(),
                                    ),
                                ],
                            }),
                            test_span(),
                        ),

                        Node::new(
                            NodeKind::Mutate(Mutate {
                                name: "outer_index".to_string(),
                                value: Box::new(Node::new(
                                    NodeKind::Add(
                                        Box::new(Node::new(
                                            NodeKind::Ident(
                                                "outer_index".to_string(),
                                            ),
                                            test_span(),
                                        )),
                                        Box::new(Node::new(
                                            NodeKind::Lit(
                                                Literal::Num(1),
                                            ),
                                            test_span(),
                                        )),
                                    ),
                                    test_span(),
                                )),
                            }),
                            test_span(),
                        ),
                    ],
                }),
                test_span(),
            ),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("nested loop evaluation should succeed");

    assert_eq!(
        evaluator.get("total"),
        Some(Value::Num(4)),
    );

    assert_eq!(
        evaluator.get("outer_index"),
        None,
    );

    assert_eq!(
        evaluator.get("inner_index"),
        None,
    );
}

#[test]
fn local_loop_define_rejects_visible_binding() {
    use crate::compiler::ast::{
        Define, Literal, Loop, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "count".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(100),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![
                    Node::new(NodeKind::Local(Box::new(
                        Node::new(NodeKind::Define(Define {
                            name: "count".to_string(),
                            value: Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(0),
                            ), test_span())),
                        }), test_span()),
                    )), test_span()),
                ],

                condition: Box::new(Node::new(NodeKind::Lt(
                    Box::new(Node::new(NodeKind::Ident(
                        "count".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(
                        Literal::Num(3),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Define(Define {
                        name: "count".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("local loop definition should not shadow a visible binding");

    assert!(
        error.message.contains("identifier `count` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn return_inside_loop_exits_enclosing_function() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Loop, Node, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Func(Func {
                name: "return_from_loop".to_string(),
                params: vec![],
                body: vec![
                    Node::new(NodeKind::Loop(Loop {
                        setup: vec![
                            Node::new(NodeKind::Define(Define {
                                name: "loop_index".to_string(),
                                value: Box::new(Node::new(NodeKind::Lit(
                                    Literal::Num(0),
                                ), test_span())),
                            }), test_span()),
                        ],

                        condition: Box::new(Node::new(NodeKind::Lt(
                            Box::new(Node::new(NodeKind::Ident(
                                "loop_index".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),

                        process: vec![
                            Node::new(NodeKind::Ret(Ret {
                                value: Some(Box::new(Node::new(NodeKind::Lit(
                                    Literal::Num(9),
                                ), test_span()))),
                            }), test_span()),

                            Node::new(NodeKind::Define(Define {
                                name: "loop_index".to_string(),
                                value: Box::new(Node::new(NodeKind::Add(
                                    Box::new(Node::new(NodeKind::Ident(
                                        "loop_index".to_string(),
                                    ), test_span())),
                                    Box::new(Node::new(NodeKind::Lit(
                                        Literal::Num(1),
                                    ), test_span())),
                                ), test_span())),
                            }), test_span()),
                        ],
                    }), test_span()),

                    Node::new(NodeKind::Ret(Ret {
                        value: Some(Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(99),
                        ), test_span()))),
                    }), test_span()),
                ],
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Call(Call {
                    callee: Box::new(Node::new(NodeKind::Ident(
                        "return_from_loop".to_string(),
                    ), test_span())),
                    args: vec![],
                }), test_span())),
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("return should propagate out of the loop and function");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(9)),
    );

    assert_eq!(
        evaluator.get("loop_index"),
        None,
    );
}

#[test]
fn loop_copy_rejects_existing_destination_binding() {
    use crate::compiler::ast::{
        Copy, Define, Literal, Loop, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(10),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "snapshot".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![],

                condition: Box::new(Node::new(NodeKind::Lt(
                    Box::new(Node::new(NodeKind::Ident(
                        "snapshot".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(
                        Literal::Num(10),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Copy(Copy {
                        name: "snapshot".to_string(),
                        target: "source".to_string(),
                    }), test_span()),

                    Node::new(NodeKind::Define(Define {
                        name: "source".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(20),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("loop copy should not reuse an existing destination binding");

    assert!(
        error.message.contains("identifier `snapshot` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn loop_bind_rejects_existing_destination_binding() {
    use crate::compiler::ast::{
        Bind, Define, Literal, Loop, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(10),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "alias".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "count".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![],

                condition: Box::new(Node::new(NodeKind::Lt(
                    Box::new(Node::new(NodeKind::Ident(
                        "count".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(
                        Literal::Num(1),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Bind(Bind {
                        name: "alias".to_string(),
                        target: "source".to_string(),
                    }), test_span()),

                    Node::new(NodeKind::Define(Define {
                        name: "source".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(20),
                        ), test_span())),
                    }), test_span()),

                    Node::new(NodeKind::Define(Define {
                        name: "count".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("loop bind should not reuse an existing destination binding");

    assert!(
        error.message.contains("identifier `alias` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn loop_guard_rejects_existing_target_binding() {
    use crate::compiler::ast::{
        Define, Guard, GuardBranch, Literal, Loop, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "count".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![],

                condition: Box::new(Node::new(NodeKind::Lt(
                    Box::new(Node::new(NodeKind::Ident(
                        "count".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(
                        Literal::Num(1),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Guard(Guard {
                        target: "result".to_string(),
                        branches: vec![
                            GuardBranch {
                                expr: Node::new(NodeKind::Lit(
                                    Literal::Num(0),
                                ), test_span()),
                            },
                            GuardBranch {
                                expr: Node::new(NodeKind::Lit(
                                    Literal::Num(7),
                                ), test_span()),
                            },
                        ],
                    }), test_span()),

                    Node::new(NodeKind::Define(Define {
                        name: "count".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("loop guard should not reuse an existing target binding");

    assert!(
        error.message.contains("identifier `result` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn local_loop_copy_rejects_visible_binding() {
    use crate::compiler::ast::{
        Copy, Define, Literal, Loop, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(10),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "snapshot".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(99),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "count".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![
                    Node::new(NodeKind::Local(Box::new(
                        Node::new(NodeKind::Copy(Copy {
                            name: "snapshot".to_string(),
                            target: "source".to_string(),
                        }), test_span()),
                    )), test_span()),
                ],

                condition: Box::new(Node::new(NodeKind::Lt(
                    Box::new(Node::new(NodeKind::Ident(
                        "count".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(
                        Literal::Num(1),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Define(Define {
                        name: "snapshot".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(20),
                        ), test_span())),
                    }), test_span()),

                    Node::new(NodeKind::Define(Define {
                        name: "count".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("local loop copy should not reuse a visible binding");

    assert!(
        error.message.contains("identifier `snapshot` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn local_loop_bind_rejects_visible_alias() {
    use crate::compiler::ast::{
        Bind, Define, Literal, Loop, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "source".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(10),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "alias".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(99),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "count".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![
                    Node::new(NodeKind::Local(Box::new(
                        Node::new(NodeKind::Bind(Bind {
                            name: "alias".to_string(),
                            target: "source".to_string(),
                        }), test_span()),
                    )), test_span()),
                ],

                condition: Box::new(Node::new(NodeKind::Lt(
                    Box::new(Node::new(NodeKind::Ident(
                        "count".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(
                        Literal::Num(1),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Define(Define {
                        name: "alias".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(20),
                        ), test_span())),
                    }), test_span()),

                    Node::new(NodeKind::Define(Define {
                        name: "count".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("local loop bind should not reuse a visible binding");

    assert!(
        error.message.contains("identifier `alias` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn local_loop_guard_rejects_visible_binding() {
    use crate::compiler::ast::{
        Define, Guard, GuardBranch, Literal, Loop, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(99),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "count".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![
                    Node::new(NodeKind::Local(Box::new(
                        Node::new(NodeKind::Guard(Guard {
                            target: "result".to_string(),
                            branches: vec![
                                GuardBranch {
                                    expr: Node::new(NodeKind::Lit(
                                        Literal::Num(0),
                                    ), test_span()),
                                },
                                GuardBranch {
                                    expr: Node::new(NodeKind::Lit(
                                        Literal::Num(7),
                                    ), test_span()),
                                },
                            ],
                        }), test_span()),
                    )), test_span()),
                ],

                condition: Box::new(Node::new(NodeKind::Lt(
                    Box::new(Node::new(NodeKind::Ident(
                        "count".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(
                        Literal::Num(1),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Define(Define {
                        name: "result".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(20),
                        ), test_span())),
                    }), test_span()),

                    Node::new(NodeKind::Define(Define {
                        name: "count".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("local loop guard should not reuse a visible binding");

    assert!(
        error.message.contains("identifier `result` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn local_empty_loop_definition_rejects_visible_binding() {
    use crate::compiler::ast::{
        Define, DefineEmpty, Literal, Loop, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::new(NodeKind::Define(Define {
                name: "value".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(99),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Define(Define {
                name: "count".to_string(),
                value: Box::new(Node::new(NodeKind::Lit(
                    Literal::Num(0),
                ), test_span())),
            }), test_span()),

            Node::new(NodeKind::Loop(Loop {
                setup: vec![
                    Node::new(NodeKind::Local(Box::new(
                        Node::new(NodeKind::DefineEmpty(DefineEmpty {
                            name: "value".to_string(),
                        }), test_span()),
                    )), test_span()),
                ],

                condition: Box::new(Node::new(NodeKind::Lt(
                    Box::new(Node::new(NodeKind::Ident(
                        "count".to_string(),
                    ), test_span())),
                    Box::new(Node::new(NodeKind::Lit(
                        Literal::Num(1),
                    ), test_span())),
                ), test_span())),

                process: vec![
                    Node::new(NodeKind::Define(Define {
                        name: "value".to_string(),
                        value: Box::new(Node::new(NodeKind::Lit(
                            Literal::Num(20),
                        ), test_span())),
                    }), test_span()),

                    Node::new(NodeKind::Define(Define {
                        name: "count".to_string(),
                        value: Box::new(Node::new(NodeKind::Add(
                            Box::new(Node::new(NodeKind::Ident(
                                "count".to_string(),
                            ), test_span())),
                            Box::new(Node::new(NodeKind::Lit(
                                Literal::Num(1),
                            ), test_span())),
                        ), test_span())),
                    }), test_span()),
                ],
            }), test_span()),
        ],
    };

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("local loop definition should not shadow a visible binding");

    assert!(
        error.message.contains("identifier `value` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn undeclared_identifier_uses_parsed_source_span() {
    use crate::compiler::error::Span;
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = "result = missing_value + 1;";

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("undeclared identifier should produce a diagnostic");

    assert_eq!(
        err.span,
        Span {
            start: 9,
            end: 22,
        },
    );
}

#[test]
fn division_by_zero_uses_rhs_source_span() {
    use crate::compiler::error::Span;
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = "result = 10 / 0;";

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("division by zero should produce a diagnostic");

    assert_eq!(err.message, "division by zero");

    assert_eq!(
        err.span,
        Span {
            start: 14,
            end: 15,
        },
    );
}

#[test]
fn modulo_by_zero_uses_rhs_source_span() {
    use crate::compiler::error::Span;
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = "result = 10 % 0;";

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("modulo by zero should produce a diagnostic");

    assert_eq!(err.message, "modulo by zero");

    assert_eq!(
        err.span,
        Span {
            start: 14,
            end: 15,
        },
    );
}

#[test]
fn negative_box_index_uses_index_expression_span() {
    use crate::compiler::error::Span;
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = "items = :[10]:; result = items::[-1];";

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("negative Box index should produce a diagnostic");

    assert_eq!(err.message, "Box index cannot be negative");

    assert_eq!(
        err.span,
        Span {
            start: 33,
            end: 35,
        },
    );
}

#[test]
fn function_defined_inside_block_can_be_called() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        :{
            fn answer :()(
                ret 42;
            ):

            answer();
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("function defined inside a block should be callable");

    assert_eq!(
        evaluator.get("answer"),
        None,
        "block-scoped function should not survive after the block ends",
    );
}

#[test]
fn function_defined_inside_block_is_unavailable_after_block() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        :{
            fn answer :()(
                ret 42;
            ):
        }:

        result = answer();
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("block-scoped function should be unavailable after the block ends");

    assert_eq!(
        err.message,
        "undeclared identifier `answer`",
    );
}

#[test]
fn function_defined_in_block_segment_is_available_in_later_segment() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        :{
            fn answer :()(
                ret 42;
            ):
        }{
            result = answer();
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("function should remain available in later block segments");
}

#[test]
fn local_function_defined_in_block_segment_is_unavailable_in_later_segment() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        :{
            loc fn answer :()(
                ret 42;
            ):
        }{
            result = answer();
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("local function should not survive into a later block segment");

    assert_eq!(
        err.message,
        "undeclared identifier `answer`",
    );
}

#[test]
fn local_define_rejects_existing_visible_name() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        value = 10;

        :{
            loc value = 20;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("local define must not reuse an existing visible name");

    assert!(
        err.message.contains("value")
            && err.message.contains("already"),
        "unexpected diagnostic: {:?}",
        err,
    );
}

#[test]
fn local_bind_can_target_outer_binding_without_changing_outer_value() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;

        :{
            loc alias :> source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("evaluation should succeed");

    assert_eq!(evaluator.get("source"), Some(Value::Num(10)));
    assert_eq!(evaluator.get("alias"), None);
}

#[test]
fn mutate_updates_existing_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        count = 10;
        count << count + 1;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("evaluation should succeed");

    assert_eq!(evaluator.get("count"), Some(Value::Num(11)));
}

#[test]
fn mutate_through_bind_updates_shared_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;
        alias :> source;
        alias << alias + 5;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("evaluation should succeed");

    assert_eq!(evaluator.get("source"), Some(Value::Num(15)));
    assert_eq!(evaluator.get("alias"), Some(Value::Num(15)));
}

#[test]
fn local_mutate_changes_value_only_within_local_scope() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;
        observed = 0;

        :{
            loc source << source + 5;
            observed << source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("evaluation should succeed");

    assert_eq!(evaluator.get("source"), Some(Value::Num(10)));
    assert_eq!(evaluator.get("observed"), Some(Value::Num(15)));
}

#[test]
fn local_mutate_preserves_local_bind_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;
        observed = 0;

        :{
            loc alias :> source;
            loc alias << alias + 5;

            observed << source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("evaluation should succeed");

    assert_eq!(evaluator.get("source"), Some(Value::Num(10)));
    assert_eq!(evaluator.get("observed"), Some(Value::Num(15)));
    assert_eq!(evaluator.get("alias"), None);
}

#[test]
fn stone_define_rejects_later_mutation() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        stone value = 10;
        value << value + 5;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone binding must reject mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `value`"
    );

    assert_eq!(
        evaluator.get("value"),
        Some(Value::Num(10))
    );
}

#[test]
fn stone_bind_makes_shared_identity_immutable() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;
        stone alias :> source;
        source << 15;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone bind must make the shared identity immutable");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `source`"
    );

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10))
    );

    assert_eq!(
        evaluator.get("alias"),
        Some(Value::Num(10))
    );
}

#[test]
fn copy_from_stone_source_remains_mutable() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        stone source = 10;
        copy := source;
        copy << 15;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("copy of a stone binding should remain mutable");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10))
    );

    assert_eq!(
        evaluator.get("copy"),
        Some(Value::Num(15))
    );
}

#[test]
fn stone_copy_creates_immutable_fresh_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;
        stone copy := source;
        copy << 15;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone copy must reject later mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `copy`"
    );

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10))
    );

    assert_eq!(
        evaluator.get("copy"),
        Some(Value::Num(10))
    );
}

#[test]
fn stone_mutate_applies_mutation_then_makes_identity_immutable() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        count = 10;
        stone count << count + 5;
        count << count + 1;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone mutate must reject later mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `count`"
    );

    assert_eq!(
        evaluator.get("count"),
        Some(Value::Num(15))
    );
}

#[test]
fn stone_mutate_through_bind_stones_shared_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;
        alias :> source;
        stone alias << alias + 5;
        source << source + 1;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone mutate through a bind must stone the shared identity");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `source`"
    );

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(15))
    );

    assert_eq!(
        evaluator.get("alias"),
        Some(Value::Num(15))
    );
}

#[test]
fn stone_local_mutate_stones_only_localized_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;
        observed = 0;

        :{
            stone loc source << source + 5;
            observed << source;
        }:

        source << source + 1;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("outer identity should remain mutable");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(11))
    );

    assert_eq!(
        evaluator.get("observed"),
        Some(Value::Num(15))
    );
}

#[test]
fn stone_local_mutate_preserves_local_bind_identity_and_restores_outer() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;
        observed = 0;

        :{
            loc alias :> source;
            stone loc alias << alias + 5;
            observed << source;
        }:

        source << source + 1;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("outer shared identity should be restored mutable");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(11))
    );

    assert_eq!(
        evaluator.get("observed"),
        Some(Value::Num(15))
    );

    assert_eq!(
        evaluator.get("alias"),
        None
    );
}

#[test]
fn stone_global_mutate_targets_global_identity_then_stones_it() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        count = 10;

        :{
            stone glo count << count + 5;
        }:

        count << count + 1;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("global identity should become stone");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `count`"
    );

    assert_eq!(
        evaluator.get("count"),
        Some(Value::Num(15))
    );
}

#[test]
fn global_mutate_targets_global_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        count = 10;

        :{
            glo count << count + 5;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("global mutation should succeed");

    assert_eq!(
        evaluator.get("count"),
        Some(Value::Num(15))
    );
}

#[test]
fn global_mutate_rejects_stone_global_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        stone count = 10;

        :{
            glo count << count + 5;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("global mutation must respect stone");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `count`"
    );

    assert_eq!(
        evaluator.get("count"),
        Some(Value::Num(10))
    );
}

#[test]
fn global_define_creates_binding_in_global_scope() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        :{
            glo created = 42;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("global definition should succeed");

    assert_eq!(
        evaluator.get("created"),
        Some(Value::Num(42))
    );
}

#[test]
fn global_define_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        existing = 10;

        :{
            glo existing = 42;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("global definition should reject an existing visible binding");

    assert!(
        error.message.contains("identifier `existing` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn global_define_empty_creates_void_binding_in_global_scope() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        :{
            glo created =;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("global empty definition should succeed");

    assert_eq!(
        evaluator.get("created"),
        Some(Value::Void)
    );
}

#[test]
fn global_copy_creates_fresh_identity_in_global_scope() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;

        :{
            glo snapshot := source;
        }:

        source << 15;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("global copy should succeed");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(15))
    );

    assert_eq!(
        evaluator.get("snapshot"),
        Some(Value::Num(10))
    );
}

#[test]
fn global_copy_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        source = 10;
        snapshot = 99;

        :{
            glo snapshot := source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("global copy should reject an existing visible binding");

    assert!(
        error.message.contains("identifier `snapshot` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn global_bind_can_alias_global_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;

        :{
            glo alias :> source;
        }:

        alias << 15;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("global bind to global identity should succeed");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(15))
    );

    assert_eq!(
        evaluator.get("alias"),
        Some(Value::Num(15))
    );
}

#[test]
fn global_bind_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        source = 10;
        alias = 99;

        :{
            glo alias :> source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("global bind should reject an existing visible binding");

    assert!(
        error.message.contains("identifier `alias` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn global_bind_rejects_local_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        :{
            local_source = 10;
            glo alias :> local_source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("global bind to local identity must fail");

    assert_eq!(
        err.message,
        "cannot create global bind to local identity `local_source`"
    );
}

#[test]
fn global_bind_can_follow_local_alias_to_global_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;

        :{
            loc temporary :> source;
            glo alias :> temporary;
        }:

        alias << 15;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("global bind through local alias to global identity should succeed");

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(15))
    );

    assert_eq!(
        evaluator.get("alias"),
        Some(Value::Num(15))
    );

    assert_eq!(
        evaluator.get("temporary"),
        None
    );
}

#[test]
fn global_guard_creates_result_in_global_scope() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        :{
            candidate = 42;
            glo result ?= candidate : 0;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    evaluator
        .eval_program(&program)
        .expect("global guard should succeed");

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(42))
    );

    assert_eq!(
        evaluator.get("candidate"),
        None
    );
}

#[test]
fn global_guard_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        result = 99;

        :{
            candidate = 42;
            glo result ?= candidate : 0;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let error = evaluator
        .eval_program(&program)
        .expect_err("global guard should reject an existing visible binding");

    assert!(
        error.message.contains("identifier `result` is already defined"),
        "unexpected error: {error:?}",
    );
}

#[test]
fn stone_global_define_creates_global_immutable_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        :{
            stone glo created = 42;
        }:

        created << 43;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global definition must reject later mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `created`"
    );

    assert_eq!(
        evaluator.get("created"),
        Some(Value::Num(42))
    );
}

#[test]
fn stone_global_define_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        created = 10;

        :{
            stone glo created = 42;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global definition should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `created` is already defined"
    );
}

#[test]
fn stone_global_define_empty_creates_global_immutable_void_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        :{
            stone glo created =;
        }:

        created << 1;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global empty definition must reject later mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `created`"
    );

    assert_eq!(
        evaluator.get("created"),
        Some(Value::Void)
    );
}

#[test]
fn stone_global_define_empty_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        created = 10;

        :{
            stone glo created =;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global empty definition should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `created` is already defined"
    );
}

#[test]
fn stone_global_copy_creates_global_immutable_fresh_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;

        :{
            stone glo snapshot := source;
        }:

        snapshot << 15;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global copy must reject later mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `snapshot`"
    );

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10))
    );

    assert_eq!(
        evaluator.get("snapshot"),
        Some(Value::Num(10))
    );
}

#[test]
fn stone_global_copy_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        source = 10;
        snapshot = 99;

        :{
            stone glo snapshot := source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global copy should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `snapshot` is already defined"
    );
}

#[test]
fn stone_global_bind_stones_shared_global_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;

        :{
            stone glo alias :> source;
        }:

        source << 15;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global bind must stone the shared identity");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `source`"
    );

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10))
    );

    assert_eq!(
        evaluator.get("alias"),
        Some(Value::Num(10))
    );
}

#[test]
fn stone_global_bind_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        source = 10;
        alias = 99;

        :{
            stone glo alias :> source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global bind should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `alias` is already defined"
    );
}

#[test]
fn stone_global_guard_creates_global_immutable_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        :{
            candidate = 42;
            stone glo result ?= candidate : 0;
        }:

        result << 43;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global guard must reject later mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `result`"
    );

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(42))
    );

    assert_eq!(
        evaluator.get("candidate"),
        None
    );
}

#[test]
fn stone_global_guard_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        result = 99;

        :{
            candidate = 42;
            stone glo result ?= candidate : 0;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone global guard should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `result` is already defined"
    );
}

#[test]
fn stone_local_define_creates_immutable_local_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        :{
            stone loc created = 42;
            created << 43;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local definition must reject mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `created`"
    );

    // Segment cleanup must still remove the localized name,
    // even though evaluation exited with a diagnostic.
    assert_eq!(
        evaluator.get("created"),
        None
    );
}

#[test]
fn stone_local_define_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        created = 10;

        :{
            stone loc created = 42;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local definition should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `created` is already defined"
    );
}

#[test]
fn stone_local_define_empty_creates_immutable_local_void_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        :{
            stone loc created =;
            created << 1;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local empty definition must reject mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `created`"
    );

    assert_eq!(
        evaluator.get("created"),
        None
    );
}

#[test]
fn stone_local_define_empty_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        created = 10;

        :{
            stone loc created =;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local empty definition should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `created` is already defined"
    );
}

#[test]
fn stone_local_copy_creates_immutable_local_fresh_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;

        :{
            stone loc snapshot := source;
            snapshot << 15;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local copy must reject mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `snapshot`"
    );

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10))
    );

    assert_eq!(
        evaluator.get("snapshot"),
        None
    );
}

#[test]
fn stone_local_copy_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        source = 10;
        snapshot = 99;

        :{
            stone loc snapshot := source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local copy should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `snapshot` is already defined"
    );
}

#[test]
fn stone_local_bind_stones_local_shared_identity_only() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        source = 10;
        observed = 0;

        :{
            stone loc alias :> source;
            observed << alias;
            alias << 15;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local bind must reject mutation through local shared identity");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `alias`"
    );

    assert_eq!(
        evaluator.get("source"),
        Some(Value::Num(10))
    );

    assert_eq!(
        evaluator.get("observed"),
        Some(Value::Num(10))
    );

    assert_eq!(
        evaluator.get("alias"),
        None
    );
}

#[test]
fn stone_local_bind_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        source = 10;
        alias = 99;

        :{
            stone loc alias :> source;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local bind should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `alias` is already defined"
    );
}

#[test]
fn stone_local_guard_creates_immutable_local_identity() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        observed = 0;

        :{
            candidate = 42;
            stone loc result ?= candidate : 0;
            observed << result;
            result << 43;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local guard must reject mutation");

    assert_eq!(
        err.message,
        "cannot mutate stone binding `result`"
    );

    assert_eq!(
        evaluator.get("observed"),
        Some(Value::Num(42))
    );

    assert_eq!(
        evaluator.get("result"),
        None
    );

    assert_eq!(
        evaluator.get("candidate"),
        None
    );
}

#[test]
fn stone_local_guard_rejects_existing_visible_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;

    let src = r#"
        result = 99;

        :{
            candidate = 42;
            stone loc result ?= candidate : 0;
        }:
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("stone local guard should reject an existing visible binding");

    assert_eq!(
        err.message,
        "identifier `result` is already defined"
    );
}

#[test]
fn define_rejects_existing_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        value = 10;
        value = 20;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("Define must reject an existing binding");

    assert_eq!(
        err.message,
        "identifier `value` is already defined"
    );

    assert_eq!(
        evaluator.get("value"),
        Some(Value::Num(10))
    );
}

#[test]
fn define_empty_rejects_existing_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        value = 10;
        value =;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("DefineEmpty must reject an existing binding");

    assert_eq!(
        err.message,
        "identifier `value` is already defined"
    );

    assert_eq!(
        evaluator.get("value"),
        Some(Value::Num(10))
    );
}

#[test]
fn guard_rejects_existing_target_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        result = 10;
        result ?= 42 : 0;
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("Guard must reject an existing target binding");

    assert_eq!(
        err.message,
        "identifier `result` is already defined"
    );

    assert_eq!(
        evaluator.get("result"),
        Some(Value::Num(10))
    );
}

#[test]
fn function_definition_rejects_existing_binding() {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::parser::Parser;
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let src = r#"
        thing = 10;
        fn thing :()():
    "#;

    let tokens = Lexer::new(src)
        .tokenize()
        .expect("lexing should succeed");

    let mut parser = Parser::new(&tokens);

    let program = parser
        .parse_program()
        .expect("parsing should succeed");

    let mut evaluator = Evaluator::new();

    let err = evaluator
        .eval_program(&program)
        .expect_err("function definition must reject an existing binding");

    assert_eq!(
        err.message,
        "identifier `thing` is already defined"
    );

    assert_eq!(
        evaluator.get("thing"),
        Some(Value::Num(10))
    );
}