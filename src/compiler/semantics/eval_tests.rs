use crate::compiler::ast::{Guard, GuardBranch, Literal, Node};
use crate::compiler::semantics::eval::Evaluator;
use crate::compiler::semantics::value::Value;

fn branch(v: Literal) -> GuardBranch {
    GuardBranch {
        expr: Node::Lit(v),
    }
}

#[test]
fn guard_assigns_first_truthy_branch() {
    let node = Node::Guard(Guard {
        target: "x".into(),
        branches: vec![
            branch(Literal::Flag(false)),
            branch(Literal::Num(1)),
            branch(Literal::Num(2)),
        ],
    });

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
    let node = Node::Guard(Guard {
        target: "x".into(),
        branches: vec![
            branch(Literal::Void),
            branch(Literal::Num(0)),
            branch(Literal::Text("".into())),
            branch(Literal::Text("ok".into())),
        ],
    });

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
    let node = Node::Guard(Guard {
        target: "x".into(),
        branches: vec![
            branch(Literal::Flag(false)),
            branch(Literal::Num(0)),
            branch(Literal::Text("".into())),
        ],
    });

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
    let node = Node::Guard(Guard {
        target: "x".into(),
        branches: vec![branch(Literal::Num(5))],
    });

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
    let node = Node::Guard(Guard {
        target: "x".into(),
        branches: vec![branch(Literal::Num(0))],
    });

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
            Node::Func(Func {
                name: "identity".to_string(),
                params: vec![Param {
                    name: "value".to_string(),
                    default: None,
                }],
                body: vec![Node::Ret(Ret {
                    value: Some(Box::new(Node::Ident(
                        "value".to_string(),
                    ))),
                })],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident(
                        "identity".to_string(),
                    )),
                    args: vec![Node::Lit(Literal::Num(7))],
                })),
            }),
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
            Node::Func(Func {
                name: "noop".to_string(),
                params: vec![],
                body: vec![],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("noop".to_string())),
                    args: vec![],
                })),
            }),
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
            Node::Func(Func {
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
                body: vec![Node::Ret(Ret {
                    value: Some(Box::new(Node::Ident(
                        "b".to_string(),
                    ))),
                })],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident(
                        "second".to_string(),
                    )),
                    args: vec![
                        Node::Lit(Literal::Num(3)),
                        Node::Lit(Literal::Num(4)),
                    ],
                })),
            }),
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
            Node::Func(Func {
                name: "identity".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: Some(Node::Lit(Literal::Num(42))),
                    },
                ],
                body: vec![Node::Ret(Ret {
                    value: Some(Box::new(Node::Ident(
                        "value".to_string(),
                    ))),
                })],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident(
                        "identity".to_string(),
                    )),
                    args: vec![],
                })),
            }),
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
            Node::Func(Func {
                name: "identity".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: Some(Node::Lit(Literal::Num(42))),
                    },
                ],
                body: vec![Node::Ret(Ret {
                    value: Some(Box::new(Node::Ident(
                        "value".to_string(),
                    ))),
                })],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident(
                        "identity".to_string(),
                    )),
                    args: vec![
                        Node::Lit(Literal::Num(7)),
                    ],
                })),
            }),
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
            Node::Func(Func {
                name: "identity".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: None,
                    },
                ],
                body: vec![],
            }),

            Node::Call(Call {
                callee: Box::new(Node::Ident(
                    "identity".to_string(),
                )),
                args: vec![],
            }),
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
            Node::Func(Func {
                name: "identity".to_string(),
                params: vec![Param {
                    name: "value".to_string(),
                    default: None,
                }],
                body: vec![],
            }),

            Node::Call(Call {
                callee: Box::new(Node::Ident(
                    "identity".to_string(),
                )),
                args: vec![
                    Node::Lit(Literal::Num(1)),
                    Node::Lit(Literal::Num(2)),
                ],
            }),
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
            Node::Call(Call {
                callee: Box::new(Node::Lit(
                    Literal::Num(7),
                )),
                args: vec![],
            }),
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
fn function_return_stops_remaining_body_execution() {
    use crate::compiler::ast::{
        Call, Define, Func, Literal, Node, Program, Ret,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::Func(Func {
                name: "early_return".to_string(),
                params: vec![],
                body: vec![
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Lit(
                            Literal::Num(1),
                        ))),
                    }),

                    Node::Define(Define {
                        name: "should_not_exist".to_string(),
                        value: Box::new(Node::Lit(
                            Literal::Num(2),
                        )),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident(
                        "early_return".to_string(),
                    )),
                    args: vec![],
                })),
            }),
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
            Node::Func(Func {
                name: "return_void".to_string(),
                params: vec![],
                body: vec![
                    Node::Ret(Ret {
                        value: None,
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident(
                        "return_void".to_string(),
                    )),
                    args: vec![],
                })),
            }),
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
            Node::Func(Func {
                name: "nested_return".to_string(),
                params: vec![],
                body: vec![
                    Node::Block(Block {
                        segments: vec![
                            BlockSegment {
                                nodes: vec![
                                    Node::Ret(Ret {
                                        value: Some(Box::new(Node::Lit(
                                            Literal::Num(9),
                                        ))),
                                    }),
                                ],
                            },
                        ],
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident(
                        "nested_return".to_string(),
                    )),
                    args: vec![],
                })),
            }),
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
            Node::Func(Func {
                name: "scoped".to_string(),
                params: vec![
                    Param {
                        name: "input".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::Define(Define {
                        name: "local_value".to_string(),
                        value: Box::new(Node::Lit(
                            Literal::Num(9),
                        )),
                    }),
                ],
            }),

            Node::Call(Call {
                callee: Box::new(Node::Ident(
                    "scoped".to_string(),
                )),
                args: vec![
                    Node::Lit(Literal::Num(4)),
                ],
            }),
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
                    Node::Define(Define {
                        name: "local_value".to_string(),
                        value: Box::new(Node::Lit(
                            Literal::Num(9),
                        )),
                    }),
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

    let identity = Node::Func(Func {
        name: "identity".to_string(),
        params: vec![
            Param {
                name: "value".to_string(),
                default: None,
            },
        ],
        body: vec![
            Node::Ret(Ret {
                value: Some(Box::new(Node::Ident(
                    "value".to_string(),
                ))),
            }),
        ],
    });

    let inner_call = Node::Call(Call {
        callee: Box::new(Node::Ident(
            "identity".to_string(),
        )),
        args: vec![
            Node::Lit(Literal::Num(7)),
        ],
    });

    let outer_call = Node::Call(Call {
        callee: Box::new(Node::Ident(
            "identity".to_string(),
        )),
        args: vec![inner_call],
    });

    let program = Program {
        nodes: vec![
            identity,

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(outer_call),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Add(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("sum".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(2)),
                        Node::Lit(Literal::Num(3)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Sub(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),
            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("difference".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(7)),
                        Node::Lit(Literal::Num(2)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Mul(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("product".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(7)),
                        Node::Lit(Literal::Num(3)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Div(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("quotient".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(21)),
                        Node::Lit(Literal::Num(3)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Mod(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("remainder".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(22)),
                        Node::Lit(Literal::Num(5)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Div(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),
            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("quotient".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(7)),
                        Node::Lit(Literal::Num(0)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Mod(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),
            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("remainder".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(7)),
                        Node::Lit(Literal::Num(0)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Eq(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("equals".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(5)),
                        Node::Lit(Literal::Num(5)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Ne(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("not_equals".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(5)),
                        Node::Lit(Literal::Num(6)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Lt(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("less_than".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(4)),
                        Node::Lit(Literal::Num(7)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Le(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("less_than_or_equal".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(7)),
                        Node::Lit(Literal::Num(7)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Gt(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("greater_than".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(9)),
                        Node::Lit(Literal::Num(4)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Ge(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("greater_than_or_equal".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(9)),
                        Node::Lit(Literal::Num(9)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::And(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("logical_and".to_string())),
                    args: vec![
                        Node::Lit(Literal::Flag(true)),
                        Node::Lit(Literal::Flag(true)),
                    ],
                })),
            }),
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
            Node::Func(Func {
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
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Or(
                            Box::new(Node::Ident("a".to_string())),
                            Box::new(Node::Ident("b".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("logical_or".to_string())),
                    args: vec![
                        Node::Lit(Literal::Flag(false)),
                        Node::Lit(Literal::Flag(true)),
                    ],
                })),
            }),
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
            Node::Func(Func {
                name: "logical_not".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Not(
                            Box::new(Node::Ident("value".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("logical_not".to_string())),
                    args: vec![
                        Node::Lit(Literal::Flag(false)),
                    ],
                })),
            }),
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
            Node::Func(Func {
                name: "negate".to_string(),
                params: vec![
                    Param {
                        name: "value".to_string(),
                        default: None,
                    },
                ],
                body: vec![
                    Node::Ret(Ret {
                        value: Some(Box::new(Node::Neg(
                            Box::new(Node::Ident("value".to_string())),
                        ))),
                    }),
                ],
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Call(Call {
                    callee: Box::new(Node::Ident("negate".to_string())),
                    args: vec![
                        Node::Lit(Literal::Num(42)),
                    ],
                })),
            }),
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
            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Lit(crate::compiler::ast::Literal::Void)),
                    Box::new(Node::Ident("anything".to_string())),
                )),
            }),
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
            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Lit(crate::compiler::ast::Literal::Void)),
                    Box::new(Node::Ident("anything".to_string())),
                )),
            }),
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
        nodes: vec![Node::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::Get(
                Box::new(Node::Lit(Literal::Flag(true))),
                Box::new(Node::Ident("anything".to_string())),
            )),
        })],
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
        nodes: vec![Node::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::Has(
                Box::new(Node::Lit(Literal::Flag(true))),
                Box::new(Node::Ident("anything".to_string())),
            )),
        })],
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
        nodes: vec![Node::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::Get(
                Box::new(Node::Lit(Literal::Num(123))),
                Box::new(Node::Ident("anything".to_string())),
            )),
        })],
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
        nodes: vec![Node::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::Has(
                Box::new(Node::Lit(Literal::Num(123))),
                Box::new(Node::Ident("anything".to_string())),
            )),
        })],
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
        nodes: vec![Node::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::Get(
                Box::new(Node::Lit(Literal::Text("hello".to_string()))),
                Box::new(Node::Ident("anything".to_string())),
            )),
        })],
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
        nodes: vec![Node::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::Has(
                Box::new(Node::Lit(Literal::Text("hello".to_string()))),
                Box::new(Node::Ident("anything".to_string())),
            )),
        })],
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
        nodes: vec![Node::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::Get(
                Box::new(Node::Lit(Literal::Dec("12.34".to_string()))),
                Box::new(Node::Ident("anything".to_string())),
            )),
        })],
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
        nodes: vec![Node::Define(Define {
            name: "result".to_string(),
            value: Box::new(Node::Has(
                Box::new(Node::Lit(Literal::Dec("12.34".to_string()))),
                Box::new(Node::Ident("anything".to_string())),
            )),
        })],
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
            Node::Define(Define {
                name: "x".to_string(),
                value: Box::new(Node::Lit(Literal::Void)),
            }),
            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Ident("x".to_string())),
                    Box::new(Node::Ident("anything".to_string())),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Num(1)),
                        Node::Lit(Literal::Num(2)),
                        Node::Lit(Literal::Num(3)),
                    ],
                })),
            }),
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
            Node::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )),
                        },
                        BagEntry {
                            name: "level".to_string(),
                            value: Node::Lit(Literal::Num(42)),
                        },
                        BagEntry {
                            name: "active".to_string(),
                            value: Node::Lit(Literal::Flag(true)),
                        },
                    ],
                })),
            }),
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
            Node::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )),
                        },
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Ident("player".to_string())),
                    Box::new(Node::Ident("name".to_string())),
                )),
            }),
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
            Node::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Ident("player".to_string())),
                    Box::new(Node::Ident("missing".to_string())),
                )),
            }),
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
            Node::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )),
                        },
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Ident("player".to_string())),
                    Box::new(Node::Ident("name".to_string())),
                )),
            }),
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
            Node::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Ident("player".to_string())),
                    Box::new(Node::Ident("missing".to_string())),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Text("first".to_string())),
                        Node::Lit(Literal::Text("second".to_string())),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Num(1)),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Num(10)),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Num(4)),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Void),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Num(0)),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Num(0)),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Bag(BagLiteral {
                            entries: vec![
                                BagEntry {
                                    name: "name".to_string(),
                                    value: Node::Lit(Literal::Text(
                                        "Rusty".to_string(),
                                    )),
                                },
                            ],
                        }),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Get(
                        Box::new(Node::Ident("data".to_string())),
                        Box::new(Node::Index(Box::new(
                            Node::Lit(Literal::Num(0)),
                        ))),
                    )),
                    Box::new(Node::Ident("name".to_string())),
                )),
            }),
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
fn get_traverses_from_bag_into_nested_box() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, BoxLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;
    use crate::compiler::semantics::value::Value;

    let program = Program {
        nodes: vec![
            Node::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "items".to_string(),
                            value: Node::Box(BoxLiteral {
                                values: vec![
                                    Node::Lit(Literal::Text(
                                        "first".to_string(),
                                    )),
                                    Node::Lit(Literal::Text(
                                        "second".to_string(),
                                    )),
                                ],
                            }),
                        },
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Get(
                        Box::new(Node::Ident("data".to_string())),
                        Box::new(Node::Ident("items".to_string())),
                    )),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Num(1)),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Bag(BagLiteral {
                            entries: vec![
                                BagEntry {
                                    name: "name".to_string(),
                                    value: Node::Lit(Literal::Text(
                                        "Rusty".to_string(),
                                    )),
                                },
                            ],
                        }),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Get(
                        Box::new(Node::Ident("data".to_string())),
                        Box::new(Node::Index(Box::new(
                            Node::Lit(Literal::Num(0)),
                        ))),
                    )),
                    Box::new(Node::Ident("name".to_string())),
                )),
            }),
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
            Node::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "items".to_string(),
                            value: Node::Box(BoxLiteral {
                                values: vec![
                                    Node::Lit(Literal::Num(10)),
                                    Node::Lit(Literal::Num(20)),
                                ],
                            }),
                        },
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Get(
                        Box::new(Node::Ident("data".to_string())),
                        Box::new(Node::Ident("items".to_string())),
                    )),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Num(1)),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Num(1)),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Ident("name".to_string())),
                )),
            }),
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
            Node::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )),
                        },
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Ident("player".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Num(0)),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Num(1)),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Ident("name".to_string())),
                )),
            }),
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
fn has_rejects_indexed_selector_on_bag() {
    use crate::compiler::ast::{
        BagEntry, BagLiteral, Define, Literal, Node, Program,
    };
    use crate::compiler::semantics::eval::Evaluator;

    let program = Program {
        nodes: vec![
            Node::Define(Define {
                name: "player".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "name".to_string(),
                            value: Node::Lit(Literal::Text(
                                "Rusty".to_string(),
                            )),
                        },
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Ident("player".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Num(0)),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "data".to_string(),
                value: Box::new(Node::Bag(BagLiteral {
                    entries: vec![
                        BagEntry {
                            name: "items".to_string(),
                            value: Node::Box(BoxLiteral {
                                values: vec![
                                    Node::Lit(Literal::Num(10)),
                                ],
                            }),
                        },
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Get(
                        Box::new(Node::Ident("data".to_string())),
                        Box::new(Node::Ident("items".to_string())),
                    )),
                    Box::new(Node::Ident("name".to_string())),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Text("first".to_string())),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Neg(Box::new(
                            Node::Lit(Literal::Num(1)),
                        )),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Text("first".to_string())),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Get(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Text("zero".to_string())),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Num(10)),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Neg(Box::new(
                            Node::Lit(Literal::Num(1)),
                        )),
                    ))),
                )),
            }),
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
            Node::Define(Define {
                name: "items".to_string(),
                value: Box::new(Node::Box(BoxLiteral {
                    values: vec![
                        Node::Lit(Literal::Num(10)),
                    ],
                })),
            }),

            Node::Define(Define {
                name: "result".to_string(),
                value: Box::new(Node::Has(
                    Box::new(Node::Ident("items".to_string())),
                    Box::new(Node::Index(Box::new(
                        Node::Lit(Literal::Text(
                            "zero".to_string(),
                        )),
                    ))),
                )),
            }),
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