use std::collections::HashMap;

use crate::{Compiler, StringIdx};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PathIdx(usize);

impl PathIdx {
    pub fn display(&self, c: &Compiler) -> String {
        return c.paths.get(*self)
            .iter().map(|s| c.strings.get(*s).to_string())
            .collect::<Vec<String>>()
            .join("::");
    }
}


#[derive(Debug, Clone)]
pub struct PathMap {
    indices: HashMap<Box<[StringIdx]>, PathIdx>,
    paths: Vec<Box<[StringIdx]>>
}

impl PathMap {
    pub fn new() -> PathMap {
        return PathMap {
            indices: HashMap::new(),
            paths: Vec::new()
        };
    }

    pub fn insert(&mut self, path: &[StringIdx]) -> PathIdx {
        if let Some(idx) = self.indices.get(path) { return *idx; }
        let idx = PathIdx(self.paths.len());
        self.indices.insert(path.into(), idx);
        self.paths.push(path.into());
        return idx;
    }

    pub fn get<'s>(&'s self, idx: PathIdx) -> &'s [StringIdx] {
        return &self.paths[idx.0];
    }
}