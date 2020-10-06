use std::collections::HashMap;

use crate::val::Val;

#[derive(Debug, Clone)]
pub struct Comp {
    map: HashMap<String, Val>,
}

impl Comp {
    pub fn new() -> Comp {
        return Comp {
            map: HashMap::new(),
        };
    }

    pub fn set(&mut self, key: &Val, val: Val) {
        self.map.insert(key.to_ink_string(), val);
    }

    pub fn get(&self, key: &Val) -> Val {
        return match self.map.get(&key.to_ink_string()) {
            Some(val) => val.clone(),
            None => Val::Null,
        };
    }

    pub fn len(&self) -> usize {
        return self.map.len();
    }
}
