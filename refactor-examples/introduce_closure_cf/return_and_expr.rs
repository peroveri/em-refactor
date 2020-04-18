fn main() {}
fn foo() -> i32 {
    for i in 10..12 {
        let j = 
        {
            if i == 11 {
                return 20;
            }
            30
        };
    }
    return 0;
}
// Introduce closure at line 5 to 10
// The conditional return at line 7 should be preserved.