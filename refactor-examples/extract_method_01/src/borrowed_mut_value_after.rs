fn inc(j: &mut i32) {
    *j += 1;
}
pub fn main() {
    let mut i = 0;
    let j = &mut i;
    inc(j);
    println!("{}", i);
}
