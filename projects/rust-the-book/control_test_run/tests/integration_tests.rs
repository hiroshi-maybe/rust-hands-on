use control_test_run;

mod common;

// $ cargo test --test integration_test
//  - Runs the specified integration test

#[test]
fn it_adds_two_in_integration_test() {
    common::setup();
    assert_eq!(4, control_test_run::add_two(2));
}
