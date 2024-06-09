use tolac::Compiler;

fn main() {
    let mut c: Compiler = Compiler::new();
    c.parse("test.tola", String::from("\
fun main(argc c::sint, argv **c::char) {
    var my_cat Cat = Cat(c\"Cookie\", 5, 0.5);
    my_cat.feed();
}
"));
    println!("ERRORS: {:?}", c.errors);
}
