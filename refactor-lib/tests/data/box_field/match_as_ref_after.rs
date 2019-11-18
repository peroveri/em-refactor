struct S {field: Box<String>}
fn main() {
    let s = S{field: Box::new("hello".to_owned())};
    match (*s.field).as_ref() {
        "hello" => {},
        _ => {panic!();}
    }
}