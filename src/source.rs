use crate::StringIdx;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Source {
    pub file: StringIdx,
    pub start: usize,
    pub end: usize
}

impl Source {
    pub fn new(file: StringIdx, start: usize, end: usize) -> Source {
        return Source { file, start, end };
    }
    
    pub fn across(start: Source, end: Source) -> Source {
        return Source { file: start.file, start: start.start, end: end.end };
    }
}