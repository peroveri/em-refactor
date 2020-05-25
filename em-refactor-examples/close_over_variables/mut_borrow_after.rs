fn main() {
    let i = &mut 0;
    (|i: &mut i32| {
        *i += 1;
    })(i);
}