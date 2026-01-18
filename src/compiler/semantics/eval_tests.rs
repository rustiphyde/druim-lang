use crate::compiler::ast::{Expr, Literal, Stmt};
use crate::compiler::semantics::eval::Evaluator;
use crate::compiler::semantics::value::Value;

fn lit(v: Literal) -> Expr {
    Expr::Lit(v)
}

#[test]
fn guard_assigns_first_truthy_branch() {
    let stmt = Stmt::Guard {
        target: "x".into(),
        branches: vec![
            lit(Literal::Flag(false)),
            lit(Literal::Num(1)), // truthy
            lit(Literal::Num(2)),
        ],
    };

    let mut ev = Evaluator::new();
    ev.eval_stmt(&stmt);

    match ev.get("x") {
        Some(Value::Num(n)) => assert_eq!(n, 1),
        other => panic!("expected x = Num(1), got {:?}", other),
    }
}

#[test]
fn guard_skips_false_values_until_true() {
    let stmt = Stmt::Guard {
        target: "x".into(),
        branches: vec![
            lit(Literal::Emp),
            lit(Literal::Num(0)),
            lit(Literal::Text("".into())),
            lit(Literal::Text("ok".into())),
        ],
    };

    let mut ev = Evaluator::new();
    ev.eval_stmt(&stmt);

    match ev.get("x") {
        Some(Value::Text(s)) => assert_eq!(s, "ok"),
        other => panic!("expected x = Text(\"ok\"), got {:?}", other),
    }
}

#[test]
fn guard_assigns_emp_if_all_branches_false() {
    let stmt = Stmt::Guard {
        target: "x".into(),
        branches: vec![
            lit(Literal::Flag(false)),
            lit(Literal::Num(0)),
            lit(Literal::Text("".into())),
        ],
    };

    let mut ev = Evaluator::new();
    ev.eval_stmt(&stmt);

    match ev.get("x") {
        Some(Value::Emp) => {}
        other => panic!("expected x = Emp, got {:?}", other),
    }
}

#[test]
fn guard_single_branch_true() {
    let stmt = Stmt::Guard {
        target: "x".into(),
        branches: vec![
            lit(Literal::Num(5)),
        ],
    };

    let mut ev = Evaluator::new();
    ev.eval_stmt(&stmt);

    match ev.get("x") {
        Some(Value::Num(n)) => assert_eq!(n, 5),
        other => panic!("expected x = Num(5), got {:?}", other),
    }
}

#[test]
fn guard_single_branch_false_becomes_emp() {
    let stmt = Stmt::Guard {
        target: "x".into(),
        branches: vec![
            lit(Literal::Num(0)),
        ],
    };

    let mut ev = Evaluator::new();
    ev.eval_stmt(&stmt);

    match ev.get("x") {
        Some(Value::Emp) => {}
        other => panic!("expected x = Emp, got {:?}", other),
    }
}
