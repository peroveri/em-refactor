fn main() {
    (||{1})();
}

mod submod {
    fn foo() {
        (|| S)();
    }
    struct S;
}