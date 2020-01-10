struct S { field: i32, other: i32 }
fn main() {
    let s1 = S { field: 3, other: 4 };
    S { field: 1, ..s1 };
}