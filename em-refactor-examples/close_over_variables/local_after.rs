fn main() {
    let i = 0;
    (|i: i32| {
        let j = i;
        let k = j;
    })(i);
}