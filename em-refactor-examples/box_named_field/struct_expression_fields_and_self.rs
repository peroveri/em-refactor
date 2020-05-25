struct S { field: i32 }
fn main() {
    let s1 = S { field: 1};
    S {
        field: s1.field
    };
}
// Here line 5 could be either 
// field: Box::new((*s1.field))
// or 
// field: s1.field
// as creating a new box-instance negates deref-ing 