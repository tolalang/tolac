use tolac::{Compiler, Error};

fn main() {
    let mut c: Compiler = Compiler::new();
    c.parse("test.tola", String::from(r#"
fun add[T](const a T, const b T): T {
    
}
"#));
    for err in c.errors.drain(..).collect::<Vec<Error>>() {
        print!("{}", err.display(&c, true));
    }
}
























