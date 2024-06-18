use tolac::Compiler;

fn main() {
    let mut comp: Compiler = Compiler::new();
    comp.parse("test.tola", String::from(r#"

mod foo;

fun add(a u32, b u32): u32 {

}


mod bar;

use foo::*;

fun main() {
    add();
}

"#));
    comp.check_types();
    for error in comp.errors() {
        print!("{}", error.display(&comp, true));
    }
}
























