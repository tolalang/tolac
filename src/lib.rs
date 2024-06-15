use std::collections::HashMap;
use std::slice::Iter;
use lexer::Lexer;

mod error;
pub use error::*;

mod source; 
use parser::{AstNode, Parser};
pub use source::*;

mod strings;
pub use strings::*;

mod paths;
pub use paths::*;

mod frontend;
pub use frontend::*;

#[derive(Debug, Clone)]
pub struct Compiler {
    pub(crate) strings: StringMap,
    pub(crate) paths: PathMap,
    pub(crate) file_contents: HashMap<StringIdx, String>,
    parsed_files: HashMap<StringIdx, Vec<AstNode>>,
    pub(crate) errors: Vec<Error>,
    errors_shared: bool
}

impl Compiler {
    pub fn new() -> Compiler {
        return Compiler { 
            strings: StringMap::new(),
            paths: PathMap::new(),
            errors: Vec::new(),
            file_contents: HashMap::new(),
            parsed_files: HashMap::new(),
            errors_shared: false
        };
    }

    pub fn parse(&mut self, path: &str, content: String) {
        let path_i: StringIdx = self.strings.insert(path);
        if self.errors_shared {
            self.errors_shared = false;
            self.errors.clear();
        } else {
            self.errors = self.errors
                .drain(..)
                .filter(|e| e.marked.file != path_i)
                .collect();
        }
        self.file_contents.insert(path_i, content.clone());
        let lexer: Lexer = Lexer::new(path_i, content);
        let mut parser: Parser = Parser::new(self, lexer);
        let nodes: Vec<AstNode> = parser.parse_file();
        println!("{:#?}", nodes);
        println!("{:#?}", self.paths);
        println!("{:#?}", self.strings);
        self.parsed_files.insert(path_i, nodes);
    }

    pub fn errors(&self) -> Iter<Error> {
        self.errors.iter()
    }

    pub fn compile(&mut self) -> Option<()> {
        if !self.errors_shared && self.errors.len() > 0 {
            return None;
        }
        self.errors_shared = true;
        self.errors.clear();
        // type check here
        // if self.errors.len() > 0 { return None; }
        // lowering here
        // if self.errors.len() > 0 { return None; }
        // codegen here
        // if self.errors.len() > 0 { return None; }
        // ...
        return Some(());
    }
}