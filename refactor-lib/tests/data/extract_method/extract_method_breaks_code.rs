fn main() {
    use std::cmp::max;
    max(1, 2);
}
// Attempt to extract line 3 as this uses a definition which is only available inside main()
// This should currently break the code as use's inside funcitons have not been considered
// This test will have to change later when use's are supported