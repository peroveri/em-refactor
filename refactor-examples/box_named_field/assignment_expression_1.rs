struct S { field: i32 }
fn main() {
    let mut s1 = S { field: 1 };
    let i = 1;
    s1.field += 1;
    s1.field = 1;
    s1.field = i;
}