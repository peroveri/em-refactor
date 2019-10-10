use crate::refactor_args::{RefactorDefinition, SourceCodeRange};

///
/// converts an argument list to a refactoring definition
///
pub fn argument_list_to_refactor_def(args: &[String]) -> Result<RefactorDefinition, String> {
    let parser = RefactorArgsParser { args };
    parser.from_args()
}

struct RefactorArgsParser<'a> {
    args: &'a [String],
}

impl RefactorArgsParser<'_> {
    pub fn from_args(&self) -> Result<RefactorDefinition, String> {
        match self.get_param("--refactoring")? {
            "extract-method" => Ok(RefactorDefinition::ExtractMethod(
                self.parse_range()?,
                self.get_param("--new_function")?.to_owned(),
            )),
            s => Err(format!("Unknown refactoring: {}", s)),
        }
    }
    pub fn parse_range(&self) -> Result<SourceCodeRange, String> {
        let selection = self.get_param("--selection")?;
        let file = self.get_param("--file")?;
        let ints = RefactorArgsParser::get_int(selection)?;

        Ok(SourceCodeRange {
            file_name: file.to_string(),
            from: ints.0,
            to: ints.1,
        })
    }
    pub fn get_int(selection: &str) -> Result<(u32, u32), String> {
        let mut split = selection.split(':');
        if let (Some(from), Some(to)) = (split.nth(0), split.nth(0)) {
            // if let Some(to) = split.nth(0) {
            return Ok((from.parse().unwrap(), to.parse().unwrap()));
            // }
        }
        Err("Selection should be formatted as <byte_from>:<byte_to>".to_owned())
    }
    fn get_param(&self, name: &str) -> Result<&str, String> {
        for t in self.args {
            let mut s = t.split('=');
            if s.nth(0) == Some(name) {
                if let Some(r) = s.nth(0) {
                    return Ok(r);
                }
            }
        }
        Err(format!("Expected {}", name))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn refactor_def_from_args() {
        let args = vec![
            "--refactoring=extract-method".to_owned(),
            "--file=main.rs".to_owned(),
            "--selection=1:2".to_owned(),
            "--new_function=foo".to_owned(),
        ];
        let scr = SourceCodeRange {
            from: 1,
            to: 2,
            file_name: "main.rs".to_owned(),
        };
        let rd = RefactorDefinition::ExtractMethod(scr, "foo".to_owned());
        let expected = Ok(rd);

        let actual = argument_list_to_refactor_def(&args);

        assert_eq!(expected, actual);
    }
}
