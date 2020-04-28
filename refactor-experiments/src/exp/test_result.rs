use regex::Regex;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TestResults {
    pub results: Vec<TestResult>,
    pub sum: TestResult
}
impl TestResults {
    pub fn new() -> Self {
        Self {
            results: vec![],
            sum: TestResult::new(0, 0, 0, 0, 0)
        }
    }
    pub fn from(s: &str) -> std::io::Result<Self> {
        let reg = Regex::new(r"test result: [^ ]+ (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out").unwrap();
        let mut results = Self {
            results: vec![],
            sum: TestResult::new(0, 0, 0, 0, 0)
        };
        for line in s.split("\n") {
            if let Some(cap) = reg.captures(line) {
                let result = TestResult::new(
                    cap.get(1).unwrap().as_str().parse().unwrap(),
                    cap.get(2).unwrap().as_str().parse().unwrap(),
                    cap.get(3).unwrap().as_str().parse().unwrap(),
                    cap.get(4).unwrap().as_str().parse().unwrap(),
                    cap.get(5).unwrap().as_str().parse().unwrap(),
                );
                results.sum.passed += result.passed;
                results.sum.failed += result.failed;
                results.sum.ignored += result.ignored;
                results.sum.measured += result.measured;
                results.sum.filtered_out += result.filtered_out;
                results.results.push(result);
            }
        }
        Ok(results)
    }
    pub fn to_single_line(&self) -> String {
        let sum = &self.sum;
        format!("passed: {}, failed: {}, ignored: {}, measured: {}, filtered_out: {}", sum.passed, sum.failed, sum.ignored, sum.measured, sum.filtered_out)
    }
    // pub fn sum(&self) -> TestResult {
    //     let mut ret = TestResult::new(0, 0);
    //     for r in &self.results {
    //         ret.passed += r.passed;
    //         ret.failed += r.failed;
    //         ret.ignored += r.ignored;
    //         ret.measured += r.measured;
    //         ret.filtered_out += r.filtered_out;
    //     }
    //     ret
    // }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TestResult {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub measured: usize,
    pub filtered_out: usize
}
impl TestResult {
    pub fn new(passed: usize, failed: usize, ignored: usize, measured: usize, filtered_out: usize) -> Self {
        Self {
            passed,
            failed,
            ignored,
            measured,
            filtered_out
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
        let expected = TestResults {
            results: vec![TestResult::new(2, 0, 2, 0, 0), TestResult::new(5, 3, 1, 0, 0)],
            sum: TestResult::new(7, 3, 3, 0, 0)
        };
        let actual = TestResults::from(TEST_OUTPUT)?;

        assert_eq!(actual, expected);

        Ok(())
    }
}