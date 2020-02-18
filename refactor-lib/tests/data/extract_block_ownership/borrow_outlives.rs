fn main() {
    let i = 0;
    let j = &i;
    let k = j;
    k;
}
// Extract line 2-4 should fail as a borrow is used later