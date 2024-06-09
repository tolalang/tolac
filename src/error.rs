use crate::Source;

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
}