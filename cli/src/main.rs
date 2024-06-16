use tolac::Compiler;

fn main() {
    let mut comp: Compiler = Compiler::new();
    comp.parse("test.tola", String::from(r#"

mod foo;

pub exp fun test(a u32, b u32): u32 {
    return a + b;
}

mod bar;

pub exp const test u32 = 69;

"#));
    comp.check_types();
    for error in comp.errors() {
        print!("{}", error.display(&comp, true));
    }
}
























