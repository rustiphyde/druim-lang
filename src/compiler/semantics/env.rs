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

    /// Define a name in the current scope.
    ///
    /// If the name already exists in the current scope, update its
    /// existing slot so all bound aliases observe the new value.
    ///
    /// Otherwise, create a fresh slot.
    pub fn define(&mut self, name: String, value: Value) {
        let scope = self
            .scopes
            .last_mut()
            .expect("no scope");

        if let Some(slot) = scope.names.get(&name) {
            slot.borrow_mut().value = value;
            return;
        }

        let slot = Rc::new(RefCell::new(Slot { value }));
        scope.names.insert(name, slot);
    }

    /// Define through normal lexical resolution.
    ///
    /// If the name already exists in any visible scope, update the nearest
    /// visible slot so existing aliases continue to observe that value.
    ///
    /// Otherwise, create a fresh slot in the current scope.
    pub fn define_nearest_or_current(
        &mut self,
        name: String,
        value: Value,
    ) {
        if let Some(slot) = self.lookup(&name) {
            slot.borrow_mut().value = value;
            return;
        }

        self.define(name, value);
    }

    /// Lookup a name, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<SlotRef> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.names.get(name).cloned())
    }

    /// Bind a new name in the current scope to an existing slot.
    pub fn bind(
        &mut self,
        name: String,
        target: &str,
    ) -> Result<(), ()> {
        let slot = self.lookup(target).ok_or(())?;

        self.scopes
            .last_mut()
            .expect("no scope")
            .names
            .insert(name, slot);

        Ok(())
    }

    /// Bind through normal lexical resolution.
    ///
    /// If the destination name exists in a visible scope, replace the nearest
    /// visible binding with the target slot.
    ///
    /// Otherwise, create the binding in the current scope.
    pub fn bind_nearest_or_current(
        &mut self,
        name: String,
        target: &str,
    ) -> Result<(), ()> {
        let target_slot = self.lookup(target).ok_or(())?;

        if let Some(scope) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.names.contains_key(&name))
        {
            scope.names.insert(name, target_slot);
            return Ok(());
        }

        self.scopes
            .last_mut()
            .expect("no scope")
            .names
            .insert(name, target_slot);

        Ok(())
    }

    /// Copy the current value of an existing slot into a fresh slot.
    pub fn copy(
        &mut self,
        name: String,
        target: &str,
    ) -> Result<(), ()> {
        let source = self.lookup(target).ok_or(())?;
        let value = source.borrow().value.clone();

        self.define(name, value);

        Ok(())
    }

    /// Copy through normal lexical resolution.
    ///
    /// If the destination name exists in a visible scope, update the nearest
    /// visible slot with an independent snapshot of the target value.
    ///
    /// Otherwise, create the copied value in the current scope.
    pub fn copy_nearest_or_current(
        &mut self,
        name: String,
        target: &str,
    ) -> Result<(), ()> {
        let source = self.lookup(target).ok_or(())?;
        let value = source.borrow().value.clone();

        self.define_nearest_or_current(name, value);

        Ok(())
    }

    /// Assign into an existing slot (mutation).
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), ()> {
        let slot = self.lookup(name).ok_or(())?;
        slot.borrow_mut().value = value;
        Ok(())
    }

    /// Temporarily remove and return a name from the current scope.
    pub fn take_current(&mut self, name: &str) -> Option<SlotRef> {
        self.scopes
            .last_mut()
            .expect("no scope")
            .names
            .remove(name)
    }

    /// Restore a name and slot in the current scope.
    pub fn restore_current(
        &mut self,
        name: String,
        slot: SlotRef,
    ) {
        self.scopes
            .last_mut()
            .expect("no scope")
            .names
            .insert(name, slot);
    }

    /// Remove a name from the current scope only.
    pub fn remove_current(&mut self, name: &str) {
        self.scopes
            .last_mut()
            .expect("no scope")
            .names
            .remove(name);
    }

    /// Convenience for tests: get the current value (if defined).
    pub fn get_value(&self, name: &str) -> Option<Value> {
        self.lookup(name).map(|s| s.borrow().value.clone())
    }
}
