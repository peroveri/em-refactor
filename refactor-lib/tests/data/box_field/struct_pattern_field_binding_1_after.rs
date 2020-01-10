struct S { field: Box<i32> }
fn main() {
    match (S {field: Box::new(0)}) {
        S {field: a} => {
            let _: i32 = (*a);
        }, // new binding 'a'
        S {..} => {}, // Et Cetera
        S {field: _} => {}, // Wildcard
        _ => {}
    }
}