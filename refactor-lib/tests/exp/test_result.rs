use regex::Regex;
use serde::Serialize;

// Before
// For each refa
//  after

#[derive(Debug, PartialEq, Serialize)]
pub struct TestRefacoringResult {
    pub before: TestResult,
    pub candidates: Vec<Candidate>,
    // pub applied: AppliedRefactoringResult
}

// #[derive(Debug, PartialEq, Serialize)]
// pub struct Candidates {
// }

#[derive(Debug, PartialEq, Serialize)]
pub struct Candidate {
    pub refactoring: String,
    pub count: usize
}


// #[derive(Debug, PartialEq, Serialize)]
// pub struct AppliedRefactoringResult {
//     pub refactoring: String,
//     pub args: String,
//     pub tests_after: TestResult
// }


#[derive(Debug, PartialEq, Serialize)]
pub struct TestResults {
    pub results: Vec<TestResult>
}
impl TestResults {
    pub fn from(s: &str) -> std::io::Result<Self> {
        let reg = Regex::new(r"test result: [^ ]+ (\d+) passed; (\d+) failed; \d+ ignored; \d+ measured; \d+ filtered out").unwrap();
        let mut r = Self {
            results: vec![]
        };
        for line in s.split("\n") {
            if let Some(cap) = reg.captures(line) {
                r.results.push(
                    TestResult::new(
                        cap.get(1).unwrap().as_str().parse().unwrap(),
                        cap.get(2).unwrap().as_str().parse().unwrap(),
                    )
                );
            }
        }
        Ok(r)
    }
    pub fn sum(&self) -> TestResult {
        let mut ret = TestResult::new(0, 0);
        for r in &self.results {
            ret.passed += r.passed;
            ret.failed += r.failed;
            ret.ignored += r.ignored;
            ret.measured += r.measured;
            ret.filtered_out += r.filtered_out;
        }
        ret
    }
}
#[derive(Debug, PartialEq, Serialize)]
pub struct TestResult {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub measured: usize,
    pub filtered_out: usize
}
impl TestResult {
    pub fn new(passed: usize, failed: usize) -> Self {
        Self {
            passed,
            failed,
            ignored: 0,
            measured: 0,
            filtered_out: 0
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_OUTPUT: &str = r"test result: ok. 2 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out


running 6 tests
test introduce_closure_control_flow_break ... ignored
test introduce_closure_grammar_block_in_assignment ... ok
test introduce_closure_grammar_block_as_expression ... ok
test introduce_closure_grammar_block_as_statement ... ok
test introduce_closure_mutate_in_same_statement ... ok
test introduce_closure_assignment_single_variable ... ok

test result: ok. 5 passed; 3 failed; 1 ignored; 0 measured; 0 filtered out";

    #[test] 
    fn test_result_should_parse() -> std::io::Result<()> {
        let expected = vec![TestResult::new(2, 0), TestResult::new(5, 3)];
        let actual = TestResults::from(TEST_OUTPUT)?;

        assert_eq!(actual.results, expected);

        Ok(())
    }
    #[test] 
    fn test_result_should_sum() -> std::io::Result<()> {
        let expected = TestResult::new(7, 3);
        let actual = TestResults::from(TEST_OUTPUT)?.sum();

        assert_eq!(actual, expected);

        Ok(())
    }
}