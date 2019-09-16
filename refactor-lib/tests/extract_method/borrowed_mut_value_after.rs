fn inc(i: &mut i32) {
    *i += 1;
}
fn main() -> i32 {
    let mut i = 0;
    let j = &mut i;
    inc(j);
    i
}
