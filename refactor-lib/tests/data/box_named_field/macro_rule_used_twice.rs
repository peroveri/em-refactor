struct S {field: i32}
struct T {field: i32}
macro_rules! foo {
    ($a: tt, $b: tt) => {$a . $b}
}

fn main() {
    let s = S{field: 1};
    let t = T{field: 2};

    let _ : i32 = foo!(s, field);
    let _ : i32 = foo!(t, field);
}