fn inc(i: &mut i32) {
    *i += 1;
}
pub fn main() -> i32 {
    let mut i = 0;
    inc(&mut i);
    return i;
}
