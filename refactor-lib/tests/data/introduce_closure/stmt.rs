fn main() {
    let i = 0;
    {
        let i = 1;
    }
    println!("{}", i);
}