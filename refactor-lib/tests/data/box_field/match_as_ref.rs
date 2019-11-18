struct S {field: String}
fn main() {
    let s = S{field: "hello".to_owned()};
    match s.field.as_ref() {
        "hello" => {},
        _ => {panic!();}
    }
}