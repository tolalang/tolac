use tolac::{Compiler, Error};

fn main() {
    let mut c: Compiler = Compiler::new();
    c.parse("test.tola", String::from("\
mod std::math; var x u32 = 10;
"));
    for err in c.errors.drain(..).collect::<Vec<Error>>() {
        print!("{}", err.display(&c, true));
    }
}
