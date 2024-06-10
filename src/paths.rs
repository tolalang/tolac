use std::collections::HashMap;

use crate::StringIdx;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PathIdx(usize);

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