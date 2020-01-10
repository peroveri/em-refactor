struct S { field: i32, other: i32 }
fn main() {
    let s1 = S { field: 3, other: 4 };
    S { other: 1, ..s1 };
}