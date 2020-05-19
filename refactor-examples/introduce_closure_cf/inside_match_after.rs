fn main() { }
fn foo() -> i32 {
    loop {
        match (|| {
            ReturnFoo::Expr(match 10 {
                11 => return ReturnFoo::Continue(),
                _ => return ReturnFoo::Return(12)
            })
        })() {
ReturnFoo::Continue() => continue,
ReturnFoo::Expr(e) => e,
ReturnFoo::Return(e) => return e}
    }
}
enum ReturnFoo {
Continue(),
Expr(()),
Return(i32)
}