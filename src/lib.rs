use std::collections::HashMap;
use std::slice::Iter;

mod error;
pub use error::*;

mod source; 
pub use source::*;

mod strings;
pub use strings::*;

mod paths;
pub use paths::*;

mod frontend;
pub use frontend::*;

const ERR_PARSING: usize = 0;
const ERR_TYPES: usize = 1;
const ERR_CODEGEN: usize = 2;

#[derive(Debug, Clone)]
pub struct Compiler {
    pub(crate) strings: StringMap,
    pub(crate) paths: PathMap,
    pub(crate) file_contents: HashMap<StringIdx, String>,
    parsed_files: HashMap<StringIdx, Vec<AstNode>>,
    pub symbols: SymbolTable,
    pub(crate) errors: Vec<Error>,
    error_stage: usize
}

impl Compiler {
    pub fn new() -> Compiler {
        return Compiler { 
            strings: StringMap::new(),
            paths: PathMap::new(),
            file_contents: HashMap::new(),
            parsed_files: HashMap::new(),
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            error_stage: ERR_PARSING
        };
    }

    pub fn parse(&mut self, path: &str, content: String) {
        let path_i: StringIdx = self.strings.insert(path);
        if self.error_stage > ERR_PARSING {
            self.error_stage = ERR_PARSING;
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
        self.parsed_files.insert(path_i, nodes);
    }

    pub fn errors(&self) -> Iter<Error> {
        self.errors.iter()
    }

    pub fn check_types(&mut self) {
        if self.error_stage < ERR_TYPES && self.errors.len() > 0 {
            return;
        }
        self.error_stage = ERR_TYPES;
        self.errors.clear();
        let parsed_nodes: Vec<Vec<AstNode>> = self
            .parsed_files.values().cloned().collect();
        let mut symbols: SymbolTable = SymbolTable::new();
        for nodes in parsed_nodes {
            symbols.insert_file(&nodes, self);
        }
        self.symbols = symbols;
        if self.errors.len() > 0 { return; }
        expand_paths(self);
        // type check here
        // if self.errors.len() > 0 { return; }
    }

    pub fn generate_output(&mut self) -> Option<()> {
        if self.error_stage < ERR_CODEGEN && self.errors.len() > 0 {
            return None;
        }
        self.error_stage = ERR_CODEGEN;
        self.errors.clear();
        // lower to IR here
        // if self.errors.len() > 0 { return None; }
        // generate code here
        // if self.errors.len() > 0 { return None; }
        // link here
        // if self.errors.len() > 0 { return None; }
        return Some(());
    }
}