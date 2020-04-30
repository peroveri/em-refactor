fn main() {
    let i = 
({fn foo() -> i32 {let mut i = 0;
    i += 1;i}
foo})();
    print("{}", i);
}