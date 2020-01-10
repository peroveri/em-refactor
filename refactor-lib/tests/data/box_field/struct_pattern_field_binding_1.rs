struct S { field: i32 }
fn main() {
    match (S {field: 0}) {
        S {field: a} => {}, // new binding 'a'
        S {..} => {}, // Et Cetera
        S {field: _} => {}, // Wildcard
        _ => {}
    }
}