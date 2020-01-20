struct S {field: i32}
macro_rules! foo {
    ($a: tt) => {
        let a: S = S { $a: 0 };
        let b: i32 = a.$a;
    }
}

fn main() {
    foo!(field);
}