fn main() {
    for i in 10..12 {
        match (|| {
            if i == 11 {
                return ReturnFoo::Break();
            }
            print!("{}", i);
        ReturnFoo::Expr(())})() {
ReturnFoo::Break() => break,
ReturnFoo::Expr(e) => e}
    }
}
enum ReturnFoo {
Break(),
Expr(())
}
// Introduce closure at line 3 to 8
// The conditional break at line 5 should be preserved.