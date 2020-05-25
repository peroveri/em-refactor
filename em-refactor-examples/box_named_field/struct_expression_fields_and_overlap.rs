struct S { field: i32 }
fn main() {
    S { field:
        S { field: 1}.field
    };
}