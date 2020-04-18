fn main() {
    struct S;
    {}
    S{};
}
// the struct declaration at line 2 is used in line 3. If we attempt to extract line 2, 
// then the declaration must be moved either before or after the new block