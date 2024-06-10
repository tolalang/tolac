use std::ops::Range;

use crate::{Compiler, GridPos, Source};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    reason: String,
    marked: Source,
    note: String
}

impl Error {
    pub fn dynamic(reason: String, marked: Source, note: String) -> Error {
        return Error { reason, marked, note };
    }

    pub fn fixed(reason: &str, marked: Source, note: &str) -> Error {
        return Error { 
            reason: String::from(reason), 
            marked, 
            note: String::from(note) 
        };
    }

    pub fn display(&self, c: &Compiler, colored: bool) -> String {
        let style_red: &'static str = if colored { "\x1b[0;91m" } else { "" };
        let style_reset: &'static str = if colored { "\x1b[0m" } else { "" };
        let marked: Range<GridPos> = self.marked.compute_grid_pos(c);
        let file: &str = c.files.get(&self.marked.file)
            .expect("file not found!");
        let mut r: String = String::new();
        r.push_str(c.strings.get(self.marked.file));
        r.push_str(":");
        r.push_str(&(marked.start.line + 1).to_string());
        r.push_str(":");
        r.push_str(&(marked.start.column + 1).to_string());
        r.push_str(": ");
        r.push_str(style_red);
        r.push_str("error: ");
        r.push_str(style_reset);
        r.push_str(&self.reason);
        r.push_str("\n");
        let max_line_n_len = marked.end.line.to_string().len();
        let lines: Vec<&str> = file.lines()
            .take(marked.end.line + 1)
            .skip(marked.start.line)
            .collect();
        for r_line_idx in 0..lines.len() {
            let line_n = (r_line_idx + marked.start.line + 1).to_string();
            r.push_str(" ");
            r.push_str(&" ".repeat(max_line_n_len - line_n.len()));
            r.push_str(&line_n);
            r.push_str(" | ");
            r.push_str(&lines[r_line_idx]);
            r.push_str("\n");
            r.push_str(" ");
            r.push_str(&" ".repeat(max_line_n_len));
            r.push_str(" | ");
            r.push_str(style_red);
            for char_idx in 0..lines[r_line_idx].len() {
                let after_start: bool = r_line_idx > 0 
                    || char_idx >= marked.start.column;
                let before_end: bool = r_line_idx < lines.len() - 1
                    || char_idx < marked.end.column;
                if after_start && before_end {
                    r.push('^');
                } else if before_end {
                    r.push(' ');
                }
            }
            r.push_str(" ");
            r.push_str(&self.note);
            r.push_str(style_reset);
            r.push_str("\n");   
        }
        return r;
    }
}