#[derive(Copy, Clone)]
struct S(i32);
fn main(){}
// it should not be possible to introduce Box on a field when the struct has the Clone attribute
// the reason for this is that Copy doesn't allow members with Drop, which Box has