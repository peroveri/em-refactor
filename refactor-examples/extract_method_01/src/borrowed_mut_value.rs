pub fn main() -> i32 {
    let mut i = 0;
    let j = &mut i;
    *j += 1;
    i
}
