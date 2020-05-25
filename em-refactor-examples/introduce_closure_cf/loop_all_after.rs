fn foo() -> i32 {
    let _: i32 =
    loop {
        let _: i32 = match (|| {
            if 1 == 1 {
                return ReturnFoo::Continue();
            } else if 3 == 3 {
                return ReturnFoo::Break(5);
            } else if 4 == 4 {
                return ReturnFoo::Return(6);
            }
            ReturnFoo::Expr(10)
        })() {
ReturnFoo::Break(e) => break e,
ReturnFoo::Continue() => continue,
ReturnFoo::Expr(e) => e,
ReturnFoo::Return(e) => return e};

        continue;
    };
    0
}
fn main() { }
enum ReturnFoo {
Break(i32),
Continue(),
Expr(i32),
Return(i32)
}
// Introduce closure at line 4 to 13