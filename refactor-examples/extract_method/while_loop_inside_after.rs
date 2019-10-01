fn inc(i: &mut i32) {
    *i += 1;
}
fn main() {
    let mut i = 0;
    while i == 0 {
        inc(&mut i);
    }

    println!("{}", i);
}