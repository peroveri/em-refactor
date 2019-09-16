#[derive(PartialEq, Debug)]
pub struct RefactorArgs {
    pub file: String,
    pub new_function: String,
    pub refactoring: String,
    pub selection: String
}

impl RefactorArgs {
    pub fn parse(s: String) -> RefactorArgs {
        let get_param = |p| {
            for t in s.split(';') {
                let mut s = t.split('=');
                if s.nth(0) == Some(p) {
                    return s.nth(0).unwrap().to_string();
                }
            }
            "".to_string()
        };
        
        RefactorArgs {
            file: get_param("--file"),
            new_function: get_param("--new_function"),
            refactoring: get_param("--refactoring"),
            selection: get_param("--selection"),
        }
    }
}

#[test]
fn reafactor_args_parse() {
    let expected = RefactorArgs {
        file: "main.rs".to_owned(),
        new_function: "foo".to_owned(),
        refactoring: "extract".to_owned(),
        selection: "1:1,2:2".to_owned(),
    };
    let actual = RefactorArgs::parse("--file=main.rs;--selection=1:1,2:2;--refactoring=extract;--new_function=foo".to_string());

    assert_eq!(expected, actual);
}