use crate::prelude::*;

// Define a struct to represent the environment
pub struct Environment {
    scopes: Vec<FxHashMap<String, ASTValue>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![FxHashMap::default()],
        }
    }

    /// Define a variable in the current scope
    pub fn define(&mut self, name: String, value: ASTValue) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    /// Assign a value to an existing variable in the current scope or any outer scope
    pub fn assign(&mut self, name: &str, value: ASTValue) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    /// Get the value of a variable in the current scope or any outer scope
    pub fn get(&self, name: &str) -> Option<ASTValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(*value);
            }
        }
        None
    }

    /// Begin a new scope.
    pub fn begin_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    /// End the current scope.
    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
