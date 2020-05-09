fn main() { }
mod mod1 {
    fn foo() {
        let i = S;
        (|i: crate::mod1::S| {
            let j = i;
        })(i);
    }
    struct S;
}