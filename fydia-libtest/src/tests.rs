#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn read_example() {
        let mut tests = crate::Tests::from_file("./tests.toml").unwrap();
        tests.run().await;
    }
}
