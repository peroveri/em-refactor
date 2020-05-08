fn main() { }
mod mod1 {
    fn foo() {
        let i = S;
        (|| {
            let j = i;
        })();
    }
    struct S;
}