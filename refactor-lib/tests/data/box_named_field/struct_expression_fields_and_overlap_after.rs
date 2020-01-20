struct S { field: Box<i32> }
fn main() {
    S { field:
        S { field: Box::new(1)}.field
    };
}