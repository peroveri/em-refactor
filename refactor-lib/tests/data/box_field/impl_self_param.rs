struct T { field: u32 }
impl T {
    fn foo(&self) {
        let _: u32 = &self.field;
    }
}
fn main() {}