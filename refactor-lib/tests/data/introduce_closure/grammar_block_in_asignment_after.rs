fn main() {
    let _ = {
        let foo = || { 1 };
        foo();
    };
}