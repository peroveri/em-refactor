mod exp;

#[test]
#[ignore]
fn experiments_run_tests() -> std::io::Result<()> {
    exp::run_all_exp()
}
