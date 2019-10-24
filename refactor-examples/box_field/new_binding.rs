struct S {field: i32 }
fn main() {
    let s = S { field: 0 };
    if let S {field} = &s { }
}
/*
 * Here field in line 4 is a new binding
 */