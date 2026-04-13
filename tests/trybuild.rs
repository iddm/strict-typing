#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/pass_*.rs");
    t.compile_fail("tests/cases/fail_*.rs");
}
