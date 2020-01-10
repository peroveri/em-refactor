struct S { field: Box<i32>, other: i32 }
fn main() {
    let s1 = S { field: Box::new(3), other: 4 };
    S { field: Box::new(1), ..s1 };
}