fn foo() -> i32 {
    let _: i32 =
    loop {
        let _: i32 = {
            if 1 == 1 {
                continue;
            } else if 3 == 3 {
                break 5;
            } else if 4 == 4 {
                return 6;
            }
            10
        };

        continue;
    };
    0
}
fn main() { }
// Introduce closure at line 4 to 13