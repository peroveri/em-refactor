struct S(i32);
fn main() {
    let mut s = S(0);
    let mut foo = || {
        s = S(s.0 + 1);
    };
foo();
    println!("{}", s.0);
}
/*
 * Here we close 'foo' in line 3. The 'mut' modifer on foo is removed, and a parameter s is added to foo.
 * Type annotations for parameter 's' is required by the compiler.
 */