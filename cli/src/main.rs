use tolac::Compiler;

fn main() {
    let mut comp: Compiler = Compiler::new();
    comp.parse("test.tola", String::from(r#"

pub exp fun add[T](a T, b T): T {
    return a + b;
}

"#));
    comp.check_types();
    for error in comp.errors() {
        print!("{}", error.display(&comp, true));
    }
}
























