fn main() {
    (||{1})();
}

mod submod {
    fn foo() {
        ({fn foo() -> crate::submod::S {
S
}
foo})();
    }
    struct S;
}