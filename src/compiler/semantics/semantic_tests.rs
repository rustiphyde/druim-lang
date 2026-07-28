use crate::compiler::semantics::truth::{truth_of, Truth};
use crate::compiler::semantics::value::Value;
use std::collections::HashMap;

#[test]
fn flag_truth_evaluates_explicitly() {
    assert_eq!(truth_of(&Value::Flag(true)), Truth::True);
    assert_eq!(truth_of(&Value::Flag(false)), Truth::False);
}

#[test]
fn void_is_always_false() {
    assert_eq!(truth_of(&Value::Void), Truth::False);
}

#[test]
fn numeric_truth_rules() {
    assert_eq!(truth_of(&Value::Num(0)), Truth::False);
    assert_eq!(truth_of(&Value::Num(1)), Truth::True);
    assert_eq!(truth_of(&Value::Num(-1)), Truth::True);
}

#[test]
fn decimal_truth_rules() {
    assert_eq!(truth_of(&Value::Dec("0".into())), Truth::False);
    assert_eq!(truth_of(&Value::Dec("0.0".into())), Truth::False);
    assert_eq!(truth_of(&Value::Dec("00.000".into())), Truth::False);

    assert_eq!(truth_of(&Value::Dec("1.0".into())), Truth::True);
    assert_eq!(truth_of(&Value::Dec("-0.5".into())), Truth::True);
}

#[test]
fn text_truth_rules() {
    assert_eq!(truth_of(&Value::Text("".into())), Truth::False);
    assert_eq!(truth_of(&Value::Text(" ".into())), Truth::False);
    assert_eq!(truth_of(&Value::Text("   ".into())), Truth::False);
    assert_eq!(truth_of(&Value::Text("\t".into())), Truth::False);
    assert_eq!(truth_of(&Value::Text("\n".into())), Truth::False);
    assert_eq!(truth_of(&Value::Text(" \t\n ".into())), Truth::False);

    assert_eq!(truth_of(&Value::Text("a".into())), Truth::True);
    assert_eq!(truth_of(&Value::Text("0".into())), Truth::True);
    assert_eq!(truth_of(&Value::Text(" hello ".into())), Truth::True);
}

#[test]
fn box_truth_rules() {
    assert_eq!(
        truth_of(&Value::Box(vec![])),
        Truth::False,
    );

    assert_eq!(
        truth_of(&Value::Box(vec![
            Value::Num(0),
        ])),
        Truth::True,
    );
}

#[test]
fn bag_truth_rules() {
    assert_eq!(
        truth_of(&Value::Bag(HashMap::new())),
        Truth::False,
    );

    let mut entries = HashMap::new();
    entries.insert(
        "value".to_string(),
        Value::Void,
    );

    assert_eq!(
        truth_of(&Value::Bag(entries)),
        Truth::True,
    );
}