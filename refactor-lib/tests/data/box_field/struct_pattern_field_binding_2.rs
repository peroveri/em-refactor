struct S { field: i32 }
impl S {
    fn foo(s: Self) {
        match s {
            Self {field: a} => {
                let _: i32 = a;
            }
        }
    }
}
fn main() {
}