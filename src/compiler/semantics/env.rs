use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::value::Value;

#[derive(Debug, Clone)]
pub struct Slot {
    pub value: Value,
}

pub type SlotRef = Rc<RefCell<Slot>>;

#[derive(Debug, Default)]
pub struct Scope {
    names: HashMap<String, SlotRef>,
}

#[derive(Debug, Default)]
pub struct Env {
    scopes: Vec<Scope>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop().expect("scope underflow");
    }

    /// Define a new name in the current scope (creates a fresh slot).
    pub fn define(&mut self, name: String, value: Value) {
        let slot = Rc::new(RefCell::new(Slot { value }));
        self.scopes
            .last_mut()
            .expect("no scope")
            .names
            .insert(name, slot);
    }

    /// Lookup a name, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<SlotRef> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.names.get(name).cloned())
    }

    /// Copy a new name in the current scope to an existing slot (aliasing).
    pub fn copy(&mut self, name: String, target: &str) -> Result<(), ()> {
        let slot = self.lookup(target).ok_or(())?;
        self.scopes
            .last_mut()
            .expect("no scope")
            .names
            .insert(name, slot);
        Ok(())
    }

    /// Assign into an existing slot (mutation).
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), ()> {
        let slot = self.lookup(name).ok_or(())?;
        slot.borrow_mut().value = value;
        Ok(())
    }

    /// Convenience for tests: get the current value (if defined).
    pub fn get_value(&self, name: &str) -> Option<Value> {
        self.lookup(name).map(|s| s.borrow().value.clone())
    }
}
