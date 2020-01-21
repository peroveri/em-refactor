fn main() {
    let foo = || {
        1
    };
    let a = foo();
}
// Introduce closure on line 2 to 4
// 'a' is assigned to the block, and the block has the type i32 (implicit)