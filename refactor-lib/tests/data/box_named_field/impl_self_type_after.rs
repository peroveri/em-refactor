struct T { field: Box<u32> }
impl T {
    fn new(field: u32) -> Self {
        Self {
            field: Box::new(field)
        }
    }
}
fn main() {}