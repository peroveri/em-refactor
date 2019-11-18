fn main() {
    let i = 0;
    let (i, j) = {
        let i = 1;
        (i, 2)
    };
    println!("{}, {}", i, j);
}