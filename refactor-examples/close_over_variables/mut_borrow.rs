fn main() {
    let i = &mut 0;
    (|| {
        *i += 1;
    })();
}