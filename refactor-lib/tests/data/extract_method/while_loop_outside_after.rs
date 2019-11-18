fn inc(i: &mut i32) {
while *i == 0 {
        *i += 1;
    }
}
fn main() {
    let mut i = 0;
    inc(&mut i);

    println!("{}", &i);
}