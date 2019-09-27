pub fn main() {
    let mut i = 0;
    let j = &mut i;
    *j += 1;
    println!("{}", i);
}
