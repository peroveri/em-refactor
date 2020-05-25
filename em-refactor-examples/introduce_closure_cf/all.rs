fn foo() -> i32 {
    for i in 10..12 {
        let _: i32 = {
            if i == 1 {
                continue;
            } else if i == 3 {
                break;
            } else if i == 4 {
                return 1;
            }
            print!("{}", i);
            10
        };
    }
    0
}
fn main() { }
// Introduce closure at line 3 to 13