fn main() {
    let mut i = 0;
    let mut foo = || {
        i = 1;
    };
    foo();
}
// Introduce closure on line 3 to 5.
// Variable i is mutated, so the variable holding the new closure 
// must have the mut modifier