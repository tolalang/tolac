use tolac::Compiler;

fn main() {
    let mut comp: Compiler = Compiler::new();
    comp.parse("test.tola", String::from(r#"

var x u8;
var y u8 = 5;
ext var z u8;
ext var w u8 = 5;

fun a();
fun b() {}
ext fun c();
ext fun d() {}

"#));
    comp.check_types();
    for error in comp.errors() {
        print!("{}", error.display(&comp, true));
    }
}
























