#[test]
fn macros() {
    let t = trybuild::TestCases::new();
    t.pass("tests/macros/pass/*.rs");
    t.compile_fail("tests/macros/compile-fail/*.rs");
}
