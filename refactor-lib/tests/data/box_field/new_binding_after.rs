struct S {field: Box<String> }
fn main() {
    let s = S { field: Box::new("".to_owned()) };
    if let S {field} = s {
        let s2: String = (*field);
     }
}
/*
 * Here field in line 4 is a new binding
 */