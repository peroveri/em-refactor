struct S ( i32 );
fn main() {
    let s1 = S(0);
    match s1 {
        S(a) => {
            let _: i32 = a;
        },
        S(a @ _) => {},
        S(_) => {},
        S(..) => {},
    }

    match s1 {
        S(a @ _) => {},
    }
}