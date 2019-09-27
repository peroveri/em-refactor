fn succ(i: &i32) -> i32 {
let j = i + 1;
return j;
}
pub fn main() {
    let i = 0;
    let j = succ(&i);
    println!("{}", j);
}
