#[test]
fn emp_is_false() {
    assert!(!is_truthy(&Value::Emp));
}

#[test]
fn zero_is_false() {
    assert!(!is_truthy(&Value::Num(0)));
}

#[test]
fn nonzero_num_is_true() {
    assert!(is_truthy(&Value::Num(5)));
}

#[test]
fn empty_text_is_false() {
    assert!(!is_truthy(&Value::Text(String::new())));
}

#[test]
fn nonempty_text_is_true() {
    assert!(is_truthy(&Value::Text("x".into())));
}
