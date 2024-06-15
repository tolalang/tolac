use tolac::Compiler;

fn main() {
    let mut comp: Compiler = Compiler::new();
    comp.parse("test.tola", String::from(r#"

    use std::(arr, vec)::(collect, iter);

    fun main() {
        var x u32 = 5;
    }

"#));
    for error in comp.errors() {
        print!("{}", error.display(&comp, true));
    }
}
























