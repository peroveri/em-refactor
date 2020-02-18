fn main() {
    let i = 0;
    let j = &i;
    let k = j;
    k;
}
// Extract line 3-4 should be ok