use std::rc::Rc;
use std::cell::RefCell;
use rustc_hash::FxHashMap;
use tsnat_common::interner::Symbol;
use crate::value::Value;

pub struct Environment<'a> {
    pub values: FxHashMap<Symbol, Value<'a>>,
    pub parent: Option<Rc<RefCell<Environment<'a>>>>,
}

impl<'a> Environment<'a> {
    pub fn new(parent: Option<Rc<RefCell<Environment<'a>>>>) -> Self {
        Self {
            values: FxHashMap::default(),
            parent,
        }
    }

    pub fn define(&mut self, name: Symbol, value: Value<'a>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: Symbol) -> Option<Value<'a>> {
        if let Some(val) = self.values.get(&name) {
            return Some(val.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        None
    }

    pub fn assign(&mut self, name: Symbol, value: Value<'a>) -> bool {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            return true;
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }
        false
    }
}
