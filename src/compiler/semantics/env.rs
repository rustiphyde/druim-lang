use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeCommitment {
    Ordinary,
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub value: Value,
    pub stone: bool,
    pub scope: ScopeCommitment,
}

pub type SlotRef = Rc<RefCell<Slot>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvError {
    UndefinedName(String),
    StoneBinding(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefineError {
    AlreadyDefined(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyError {
    UndefinedName(String),
    AlreadyDefined(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalCopyError {
    UndefinedName(String),
    AlreadyDefined(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindError {
    UndefinedName(String),
    AlreadyDefined(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalBindError {
    UndefinedName(String),
    LocalIdentity(String),
    AlreadyDefined(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalMutateError {
    UndefinedName(String),
    StoneBinding(String),
    LocalIdentity(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalMutateError {
    UndefinedName(String),
    StoneBinding(String),
    GlobalIdentity(String),
}

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
    pub fn define(
        &mut self,
        name: String,
        value: Value,
    ) -> Result<(), DefineError> {
        if self.lookup(&name).is_some() {
            return Err(DefineError::AlreadyDefined(name));
        }

        let scope = self.scopes.last_mut().expect("no scope");

        let slot = Rc::new(RefCell::new(Slot {
            value,
            stone: false,
            scope: ScopeCommitment::Ordinary,
        }));

        scope.names.insert(name, slot);

        Ok(())
    }

    pub fn define_global(
        &mut self,
        name: String,
        value: Value,
    ) -> Result<(), DefineError> {
        if self.lookup(&name).is_some() {
            return Err(DefineError::AlreadyDefined(name));
        }

        let scope = self
            .scopes
            .first_mut()
            .expect("no global scope");

        let slot = Rc::new(RefCell::new(Slot {
            value,
            stone: false,
            scope: ScopeCommitment::Global,
        }));

        scope.names.insert(name, slot);

        Ok(())
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
    ) -> Result<(), BindError> {
        if self.lookup(&name).is_some() {
            return Err(BindError::AlreadyDefined(name));
        }

        let slot = self
            .lookup(target)
            .ok_or_else(|| BindError::UndefinedName(target.to_string()))?;

        self.scopes
            .last_mut()
            .expect("no scope")
            .names
            .insert(name, slot);

        Ok(())
    }

    pub fn bind_global(
        &mut self,
        name: String,
        target: &str,
    ) -> Result<(), GlobalBindError> {
        if self.lookup(&name).is_some() {
            return Err(GlobalBindError::AlreadyDefined(name));
        }

        let target_slot = self
            .lookup(target)
            .ok_or_else(|| {
                GlobalBindError::UndefinedName(target.to_string())
            })?;

        let is_global_identity = {
            let global = self
                .scopes
                .first()
                .expect("no global scope");

            global
                .names
                .values()
                .any(|slot| Rc::ptr_eq(slot, &target_slot))
        };

        if !is_global_identity {
            return Err(
                GlobalBindError::LocalIdentity(target.to_string()),
            );
        }

        let global = self
            .scopes
            .first_mut()
            .expect("no global scope");

        global.names.insert(name, target_slot);

        Ok(())
    }

    /// Copy the current value of an existing slot into a fresh slot.
    pub fn copy(
        &mut self,
        name: String,
        target: &str,
    ) -> Result<(), CopyError> {
        let source = self
            .lookup(target)
            .ok_or_else(|| CopyError::UndefinedName(target.to_string()))?;

        let value = source.borrow().value.clone();

        self.define(name, value)
            .map_err(|error| {
                match error {
                    DefineError::AlreadyDefined(name) => {
                        CopyError::AlreadyDefined(name)
                    }
                }
            })?;

        Ok(())
    }

    pub fn copy_global(
        &mut self,
        name: String,
        target: &str,
    ) -> Result<(), GlobalCopyError> {
        if self.lookup(&name).is_some() {
            return Err(GlobalCopyError::AlreadyDefined(name));
        }

        let source = self
            .lookup(target)
            .ok_or_else(|| GlobalCopyError::UndefinedName(target.to_string()))?;

        let value = source.borrow().value.clone();

        let scope = self
            .scopes
            .first_mut()
            .expect("no global scope");

        let slot = Rc::new(RefCell::new(Slot {
            value,
            stone: false,
            scope: ScopeCommitment::Global,
        }));

        scope.names.insert(name, slot);

        Ok(())
    }

    pub fn mark_stone(
        &mut self,
        name: &str,
    ) -> Result<(), EnvError> {
        let slot = self
            .lookup(name)
            .ok_or_else(|| EnvError::UndefinedName(name.to_string()))?;

        slot.borrow_mut().stone = true;

        Ok(())
    }

    /// Assign into an existing slot (mutation).
    pub fn assign(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<(), EnvError> {
        let slot = self
            .lookup(name)
            .ok_or_else(|| EnvError::UndefinedName(name.to_string()))?;

        if slot.borrow().stone {
            return Err(EnvError::StoneBinding(name.to_string()));
        }

        slot.borrow_mut().value = value;

        Ok(())
    }

    pub fn assign_global(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<(), GlobalMutateError> {
        let slot = self
            .lookup(name)
            .ok_or_else(|| GlobalMutateError::UndefinedName(name.to_string()))?;

        {
            let binding = slot.borrow();

            if binding.stone {
                return Err(GlobalMutateError::StoneBinding(name.to_string()));
            }

            if binding.scope == ScopeCommitment::Local {
                return Err(GlobalMutateError::LocalIdentity(name.to_string()));
            }
        }

        {
            let mut binding = slot.borrow_mut();
            binding.value = value;
            binding.scope = ScopeCommitment::Global;
        }

        for scope in self.scopes.iter_mut().skip(1) {
            scope.names.remove(name);
        }

        self.scopes
            .first_mut()
            .expect("no global scope")
            .names
            .insert(name.to_string(), slot);

        Ok(())
    }

    pub fn mark_local(
        &mut self,
        name: &str,
    ) -> Result<(), EnvError> {
        let slot = self
            .lookup(name)
            .ok_or_else(|| EnvError::UndefinedName(name.to_string()))?;

        slot.borrow_mut().scope = ScopeCommitment::Local;

        Ok(())
    }

    pub fn mark_global_stone(
        &mut self,
        name: &str,
    ) -> Result<(), EnvError> {
        let scope = self.scopes.first().expect("no global scope");

        let slot = scope
            .names
            .get(name)
            .cloned()
            .ok_or_else(|| EnvError::UndefinedName(name.to_string()))?;

        slot.borrow_mut().stone = true;

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

    pub fn localize_identity(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<Vec<(String, Option<SlotRef>)>, LocalMutateError> {
        let original = self
            .lookup(name)
            .ok_or_else(|| LocalMutateError::UndefinedName(name.to_string()))?;

        {
            let binding = original.borrow();

            if binding.stone {
                return Err(LocalMutateError::StoneBinding(name.to_string()));
            }

            if binding.scope == ScopeCommitment::Global {
                return Err(LocalMutateError::GlobalIdentity(name.to_string()));
            }
        }

        let mut names = Vec::new();

        for scope in &self.scopes {
            for (candidate, slot) in &scope.names {
                if Rc::ptr_eq(slot, &original) && !names.contains(candidate) {
                    names.push(candidate.clone());
                }
            }
        }

        let localized = Rc::new(RefCell::new(Slot {
            value,
            stone: false,
            scope: ScopeCommitment::Local,
        }));

        let current = self
            .scopes
            .last_mut()
            .expect("no scope");

        let mut previous = Vec::new();

        for alias in names {
            let old = current
                .names
                .insert(alias.clone(), localized.clone());

            previous.push((alias, old));
        }

        Ok(previous)
    }
}
