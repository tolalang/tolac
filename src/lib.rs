use std::collections::HashMap;
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
    pub strings: StringMap,
    pub paths: PathMap,
    pub files: HashMap<StringIdx, String>,
    pub errors: Vec<Error>
}

impl Compiler {
    pub fn new() -> Compiler {
        return Compiler { 
            strings: StringMap::new(),
            paths: PathMap::new(),
            errors: Vec::new(),
            files: HashMap::new()
        };
    }

    pub fn parse(&mut self, path: &str, content: String) {
        let path_i: StringIdx = self.strings.insert(path);
        self.files.insert(path_i, content.clone());
        let lexer: Lexer = Lexer::new(path_i, content);
        let mut parser: Parser = Parser::new(self, lexer);
        let nodes: Vec<AstNode> = parser.parse_file();
        println!("{:#?}", nodes); 
    }
}