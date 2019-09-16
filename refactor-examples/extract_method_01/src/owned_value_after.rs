fn succ(i: &i32) -> i32 {
    i + 1
}
pub fn main() -> i32 {
    let i = 0;
    let j = succ(&i);
    j
}
