use std::ops::Range;

use crate::{Compiler, GridPos, Source};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    pub reason: String,
    pub marked: Option<Source>
}

impl Error {
    pub fn dynamic(reason: String, marked: Source) -> Error {
        return Error { reason, marked: Some(marked) };
    }

    pub fn fixed(reason: &str, marked: Source) -> Error {
        return Error { 
            reason: String::from(reason), 
            marked: Some(marked)
        };
    }

    pub fn message(reason: String) -> Error {
        return Error { 
            reason: String::from(reason), 
            marked: None
        };
    }

    pub fn display(&self, c: &Compiler, colored: bool) -> String {
        let style_red: &'static str = if colored { "\x1b[91m" } else { "" };
        let style_gray: &'static str = if colored { "\x1b[90m" } else { "" };
        let style_bold: &'static str = if colored { "\x1b[1m" } else { "" };
        let style_reset: &'static str = if colored { "\x1b[0m" } else { "" };
        let mut r: String = String::new();
        if let Some(marked) = self.marked {
            let marked_gp: Range<GridPos> = marked.compute_grid_pos(c);
            let file: &str = c.file_contents.get(&marked.file)
                .expect("file not found!");
            r.push_str(c.strings.get(marked.file));
            r.push_str(":");
            r.push_str(&(marked_gp.start.line + 1).to_string());
            r.push_str(":");
            r.push_str(&(marked_gp.start.column + 1).to_string());
            r.push_str(": ");
            r.push_str(style_red);
            r.push_str(style_bold);
            r.push_str("error: ");
            r.push_str(style_reset);
            r.push_str(&self.reason);
            r.push_str("\n");
            let max_line_n_len = marked_gp.end.line.to_string().len();
            let lines: Vec<&str> = file.lines()
                .take(marked_gp.end.line + 1)
                .skip(marked_gp.start.line)
                .collect();
            for r_line_idx in 0..lines.len() {
                let line_n = (r_line_idx + marked_gp.start.line + 1)
                    .to_string();
                r.push_str(style_gray);
                r.push_str(" ");
                r.push_str(&" ".repeat(max_line_n_len - line_n.len()));
                r.push_str(&line_n);
                r.push_str(" | ");
                r.push_str(style_reset);
                r.push_str(&lines[r_line_idx]);
                r.push_str("\n");
                r.push_str(style_gray);
                r.push_str(" ");
                r.push_str(&" ".repeat(max_line_n_len));
                r.push_str(" | ");
                r.push_str(style_red);
                r.push_str(style_bold);
                for char_idx in 0..lines[r_line_idx].len() {
                    let after_start: bool = r_line_idx > 0 
                        || char_idx >= marked_gp.start.column;
                    let before_end: bool = r_line_idx < lines.len() - 1
                        || char_idx < marked_gp.end.column;
                    if after_start && before_end {
                        r.push('^');
                    } else if before_end {
                        r.push(' ');
                    }
                }
                r.push_str(style_reset);
                r.push_str("\n");   
            }
        } else {
            r.push_str(style_red);
            r.push_str(style_bold);
            r.push_str("error: ");
            r.push_str(style_reset);
            r.push_str(&self.reason);
            r.push_str("\n");
        }
        return r;
    }
}