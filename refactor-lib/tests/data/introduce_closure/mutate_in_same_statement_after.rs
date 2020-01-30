fn main() {
    let mut i = 0;
 
    (i = 1, {
        let mut foo = || {
            i = 2;
        };
        foo()
    });
}
// An example where the closure definition cannot be moved
// outside to the parent block because of the mutable borrow
// at line 4.