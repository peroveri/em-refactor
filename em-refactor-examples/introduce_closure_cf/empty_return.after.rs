fn main() {
    match (|| { 
        return ReturnFoo::Return();
    ReturnFoo::Expr(())}
)() {
ReturnFoo::Expr(e) => e,
ReturnFoo::Return() => return}}
enum ReturnFoo {
Expr(()),
Return()
}