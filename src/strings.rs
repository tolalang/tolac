use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StringIdx(usize);

#[derive(Debug, Clone)]
pub struct StringMap {
    indices: HashMap<Rc<str>, StringIdx>,
    strings: Vec<Rc<str>>
}

impl StringMap {
    pub fn new() -> StringMap {
        return StringMap {
            indices: HashMap::new(),
            strings: Vec::new()
        };
    }

    pub fn insert(&mut self, string: &str) -> StringIdx {
        if let Some(idx) = self.indices.get(string) { return *idx; }
        let idx = StringIdx(self.strings.len());
        let string: Rc<str> = string.into();
        self.indices.insert(Rc::clone(&string), idx);
        self.strings.push(string);
        return idx;
    }

    pub fn get<'s>(&'s self, idx: StringIdx) -> &'s str {
        return &self.strings[idx.0];
    }
}