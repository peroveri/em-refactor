fn main() {}
fn foo() -> i32 {
    for i in 10..12 {
        let j = 
        match (|| {
            if i == 11 {
                return ReturnFoo::Return(20);
            }
            ReturnFoo::Expr(30)
        })() {
ReturnFoo::Expr(e) => e,
ReturnFoo::Return(e) => return e};
    }
    return 0;
}
enum ReturnFoo {
Expr(i32),
Return(i32)
}
// Introduce closure at line 5 to 10
// The conditional return at line 7 should be preserved.