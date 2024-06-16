use tolac::Compiler;

fn main() {
    let mut comp: Compiler = Compiler::new();
    comp.parse("test.tola", String::from(r#"

mod foo;

fun add(a u32, b u32): u32 {
    return a + b;
}

fun add(a s32, b s32): s32 {
    return a + b;
}

"#));
    comp.check_types();
    for error in comp.errors() {
        print!("{}", error.display(&comp, true));
    }
}
























