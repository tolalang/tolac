use tolac::{Compiler, Error};
use std::env;
use std::fs;

fn main() {
    let mut comp: Compiler = Compiler::new();
    let mut errors: Vec<Error> = Vec::new();
    for arg in env::args().skip(1) {
        match fs::read_to_string(&arg) {
            Ok(contents) => comp.parse(&arg, contents),
            Err(reason) => {
                errors.push(Error::message(format!(
                    "the file '{}' could not be read: {}", 
                    arg, reason.to_string()
                )));
            }
        }
    }
    if errors.len() == 0 {
        comp.check_types();
        comp.generate_output();
    }
    errors.extend_from_slice(comp.errors());
    for error in errors {
        print!("{}", error.display(&comp, true));
    }
}
























