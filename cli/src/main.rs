use tolac::{Compiler, Error, Source};

fn main() {
    let mut c: Compiler = Compiler::new();
    c.parse("test.tola", String::from("\
fun main(argc c::sint, argv **c::char) {
    var my_cat Cat = Cat(c\"Cookie\\nballs\\u{1F602}\", 5, 0.5);
    my_cat.feed();
    lmfao?lol
    this is | funny
}
"));
    for err in c.errors.drain(..).collect::<Vec<Error>>() {
        print!("{}", err.display(&c, true));
    }
}
