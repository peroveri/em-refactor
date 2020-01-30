fn main() {
    let mut i = 0;
 
    (i = 1, {
        i = 2;
    });
}
// An example where the closure definition cannot be moved
// outside to the parent block because of the mutable borrow
// at line 4.