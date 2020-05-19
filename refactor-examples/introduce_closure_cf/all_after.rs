fn foo() -> i32 {
    for i in 10..12 {
        let _: i32 = match (|| {
            if i == 1 {
                return ReturnFoo::Continue();
            } else if i == 3 {
                return ReturnFoo::Break();
            } else if i == 4 {
                return ReturnFoo::Return(1);
            }
            print!("{}", i);
            ReturnFoo::Expr(10)
        })() {
ReturnFoo::Break() => break,
ReturnFoo::Continue() => continue,
ReturnFoo::Expr(e) => e,
ReturnFoo::Return(e) => return e};
    }
    0
}
fn main() { }
enum ReturnFoo {
Break(),
Continue(),
Expr(i32),
Return(i32)
}
// Introduce closure at line 3 to 13