struct S {i: i32}
pub fn main() {
    let i = S {i: 0};
    let j = &i;
    println!("{}", j.i);
}
