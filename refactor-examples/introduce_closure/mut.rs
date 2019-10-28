fn main() {
    let mut i = 0;
    {
        i += 1;
    };
    println!("{}", i);
}
/*
 * Here we extract line 3-5. I is mutated in the block, so the resulting closure must have the mut modifier
 */