use crate::LibTest;

#[test]
pub fn read_tests() {
    let file = include_str!("../../fydia-router/tests.json");

    fydia_utils::serde_json::from_str::<LibTest>(file)
        .unwrap()
        .run_tests();
}
