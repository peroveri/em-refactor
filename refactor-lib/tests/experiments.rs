mod exp;

#[test]
#[ignore]
fn experiments_run_tests() -> std::io::Result<()> {
    // exp::run_all_exp("box-named-field")
    // TODO: we probably don't want to query candidates for the tests
    exp::run_all_exp("extract-block")
}
