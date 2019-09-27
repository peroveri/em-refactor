fn succ(i: &i32) -> i32 {
    return i + 1;
}
pub fn main() {
    let i = 0;
    let j = succ(&i);
    println!("{}", j);
}
