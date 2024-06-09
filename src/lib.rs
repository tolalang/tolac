use std::collections::HashMap;

mod error;
pub use error::*;

mod source; 
pub use source::*;

mod strings;
pub use strings::*;

mod frontend;
pub use frontend::*;

#[derive(Debug, Clone)]
pub struct Compiler {
    pub strings: StringMap,
    pub files: HashMap<StringIdx, String>,
    pub errors: Vec<Error>
}

impl Compiler {
    pub fn new() -> Compiler {
        return Compiler { 
            strings: StringMap::new(),
            errors: Vec::new(),
            files: HashMap::new()
        };
    }

    pub fn parse(&mut self, path: &str, content: String) {
        let path_i: StringIdx = self.strings.insert(path);
        self.files.insert(path_i, content.clone());
        let mut lexer: Lexer = Lexer::new(path_i, content);
        // this is for testing
        loop {
            let t: Token = lexer.next(self);
            if t.t == TokenType::EndOfFile { break; }
            println!("{}", self.strings.get(t.content));
        }
    }
}