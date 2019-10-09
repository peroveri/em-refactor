#[derive(PartialEq, Debug)]
pub struct RefactorArgs {
    pub file: String,
    pub new_function: String,
    pub refactoring: String,
    pub selection: String
}

impl RefactorArgs {
    pub fn parse(s: String) -> Result<RefactorArgs, String> {
        let get_param = |p| {
            for t in s.split(';') {
                let mut s = t.split('=');
                if s.nth(0) == Some(p) {
                    return Ok(s.nth(0).unwrap().to_string());
                }
            }
            Err(format!("Expected {}", p))
        };

        let refactoring = get_param("--refactoring")?;

        if refactoring != "extract-method" { // TODO: move this somewhere else, but it should happen early
            return Err(format!("Unknown refactoring: {}", refactoring));
        }
        
        Ok(RefactorArgs {
            refactoring,
            file: get_param("--file")?,
            new_function: get_param("--new_function")?,
            selection: get_param("--selection")?,
        })
    }
}

#[test]
fn reafactor_args_parse() {
    let expected = Ok(RefactorArgs {
        file: "main.rs".to_owned(),
        new_function: "foo".to_owned(),
        refactoring: "extract-method".to_owned(),
        selection: "1:1,2:2".to_owned(),
    });
    let actual = RefactorArgs::parse("--file=main.rs;--selection=1:1,2:2;--refactoring=extract-method;--new_function=foo".to_string());

    assert_eq!(expected, actual);
}