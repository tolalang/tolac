use std::ops::Range;
use crate::{Compiler, StringIdx};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub line: usize,
    pub column: usize
}

impl GridPos {
    pub fn new(line: usize, column: usize) -> GridPos {
        return GridPos { line, column };
    }
}


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

    pub fn compute_grid_pos(&self, c: &Compiler) -> Range<GridPos> {
        let file: &str = c.files.get(&self.file)
            .expect("source file must be valid");
        let mut pos: usize = 0;
        let mut curr: GridPos = GridPos::new(0, 0);
        let mut start: GridPos = GridPos::new(0, 0);
        let mut end: GridPos = GridPos::new(0, 0);
        while pos < file.len() {
            if pos == self.start { start = curr; }
            if pos == self.end { end = curr; }
            if &file[pos..pos + 1] == "\n" {
                curr.line += 1;
                curr.column = 0;
            } else {
                curr.column += 1;
            }
            pos += 1;
        }
        return start..end;
    }
}