use crate::expr::LiteralValue;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, LiteralValue>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::<String, LiteralValue>::new(),
            enclosing: None,
        }
    }

    pub fn define(&mut self, name: String, val: LiteralValue) {
        self.values.insert(name, val);
    }

    pub fn get(&self, name: &str) -> Option<LiteralValue> {
        let value = self.values.get(name);

        match (value, &self.enclosing) {
            (Some(val), _) => Some(val.clone()),
            (None, Some(env)) => env.borrow().get(name),
            (None, None) => None,
        }
    }

    pub fn assign(&mut self, name: &str, value: LiteralValue) -> bool {
        let old_value = self.values.get(name);
        match (old_value, &mut self.enclosing) {
            (Some(_), _) => {
                self.values.insert(name.to_string(), value);
                true
            }
            (None, Some(env)) => (env.borrow_mut()).assign(name, value),
            (None, None) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_init() {
        let environment = Environment::new();
    }
}
