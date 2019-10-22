struct S {
    field: Box<i32>
}
fn main() {
    let mut s = S{field: Box::new(0)};
    s.field = Box::new(1);
    println!("{}", *s.field);
}
/*
 * Here we test that line 2, 5 and 6 should be modified.
 * At line 5 and 6: The assignments should be wrapped in Box::new
 * At line 7: the field access should be wrapped in *
 */