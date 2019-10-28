fn main() {
    let mut i = 0;
    let mut foo = || {
        i += 1;
    };
foo();
    println!("{}", i);
}
/*
 * Here we extract line 3-5. I is mutated in the block, so the resulting closure must have the mut modifier
 */