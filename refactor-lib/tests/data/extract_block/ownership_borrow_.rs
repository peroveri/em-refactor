fn foo(b: bool) {
    let (mut i, mut j) = (0, 0);
    let h = if b {
        &mut i
    } else {
        &mut j
    };
    *h += 1;
}
fn main() { }
// Extract block on line 4 to 8.
// Variable 'h' borrows both i and j at line 3