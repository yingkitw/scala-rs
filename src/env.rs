use std::collections::HashMap;
use crate::value::Value;

pub struct Environment {
    scopes: Vec<HashMap<String, (Value, bool)>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: &str, value: Value, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), (value, mutable));
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some((value, _)) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if let Some((existing, mutable)) = scope.get_mut(name) {
                if *mutable {
                    *existing = value;
                    return true;
                }
                return false;
            }
        }
        false
    }

    pub fn capture(&self) -> Vec<(String, Value, bool)> {
        let mut captured = Vec::new();
        for scope in &self.scopes {
            for (name, (value, mutable)) in scope {
                captured.push((name.clone(), value.clone(), *mutable));
            }
        }
        captured
    }

    pub fn restore(&mut self, captured: &[(String, Value, bool)]) {
        for (name, value, mutable) in captured {
            self.define(name, value.clone(), *mutable);
        }
    }
}
