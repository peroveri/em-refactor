struct S { field: i32 }

impl Clone for S {
    fn clone(&self) -> Self {
        S {field: 0}
    }
}
impl Copy for S { }

fn main(){}
// it should not be possible to introduce Box on a field when the struct has the Clone attribute
// the reason for this is that Copy doesn't allow members with Drop, which Box has