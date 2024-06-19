use tolac::Compiler;

fn main() {
    let mut comp: Compiler = Compiler::new();
    comp.parse("test.tola", String::from(r#"

mod foo;

fun x() {}
fun y() {}


mod bar;

use foo::*;

fun main() {
    const x u32 = 5;
    x();
    y();
}

"#));
    comp.check_types();
    for error in comp.errors() {
        print!("{}", error.display(&comp, true));
    }
}
























