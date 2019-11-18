fn main() {
    let i = 0;
    let foo = || {
        let i = 1;
        (i, 2)
    };
let (i, j) = foo();
    println!("{}, {}", i, j);
}