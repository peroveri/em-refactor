fn main() {
    let mut i = 0;
    {
        i += 1;
    }
    println!("{}", &i);
}