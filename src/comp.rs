use std::collections::HashMap;

use crate::val::Val;

#[derive(Debug, Clone)]
pub struct Comp {
    // TODO: map needs to be Arc<HashMap> because multiple register values possibly across frames
    // should be able to refer to the same Comp value, and a uniquely owned HashMap can't emulate
    // that which means argument passing and MOV instructions currently clone the entire hashmap.
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
