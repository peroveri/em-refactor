fn foo(i: &mut i32) {
    *i += 1;
}
fn main() {
    let mut i = 0;
    {
        foo(&mut i);
    }
    println!("{}", &i);
}