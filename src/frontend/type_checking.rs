use crate::Compiler;


#[derive(Debug)]
pub struct TypeChecker<'c> {
    comp: &'c mut Compiler
}

impl <'c> TypeChecker<'c> {
    pub fn new(comp: &mut Compiler) -> TypeChecker {
        return TypeChecker {
            comp
        };
    }

    pub fn check_types(&mut self) {
        todo!("type checking")
    }
}