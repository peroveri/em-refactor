fn main() {
    let i = 0;
    let foo = || {
        let i = 1;
    };
foo();
    println!("{}", i);
}