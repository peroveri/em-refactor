struct S(i32);
fn main() {
    let s = S(0);
    let s = 
{
let s = S(1);
s};
    print!("{}", s.0);
}