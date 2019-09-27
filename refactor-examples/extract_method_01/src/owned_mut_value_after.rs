fn inc(i: &mut i32) {
*i += 1;
}
pub fn main() {
    let mut i = 0;
    inc(&mut i);
    println!("{}", i);
}
