use std::collections::HashMap;

use crate::gen::Val;

#[derive(Debug)]
pub struct Comp {
    map: HashMap<String, Val>,
}

#[allow(unused)]
impl Comp {
    pub fn set(key: &Val, val: Val) {}

    pub fn get(key: &Val) -> Val {
        return Val::Empty;
    }
}
