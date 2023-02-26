use std::{process::{Stdio, Child}, io::Read};

use crate::LibTest;

pub fn new() {}

#[tokio::test]
pub async fn read_tests() {
    let file = include_str!("../../fydia-router/tests.json");
    let mut server = std::process::Command::new("../target/debug/fydia")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = server.stdout.unwrap();
    let mut string = String::new();
    while stdout.read_to_string(&mut string).unwrap() 
    let b = fydia_utils::serde_json::from_str::<LibTest>(file).unwrap();
    println!("{:?}", b);
    b.run_tests().await;

    server.kill().unwrap();
}
