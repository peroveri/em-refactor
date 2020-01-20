#[derive(Debug)]
struct S { field: i32 }
fn main(){}
// it should be possible to box a field when the struct has the Debug attribute
// this is currently not implemented, so the test is ignored