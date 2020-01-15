fn main() {
    fn max(_: i32, _: i32) {}
    max(0, 0);
    use std::cmp::max;
    max(1, 1);
    max(2, 2);
}