fn main() { }
fn foo() -> i32 {
    loop {
        match (|| {
            match 10 {
                11 => continue,
                _ => return 12
            }
        })() {
            () => continue,
            () => return a
        }
    }
}