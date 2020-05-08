fn main() {
    let i = 0;
    (|| {
        let j = i;
        let k = j;
    })();
}