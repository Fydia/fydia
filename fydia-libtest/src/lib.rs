use std::{collections::HashMap, time::Instant};

use structures::{Config, Test};
use toml::Value;

mod structures;
mod tests;

#[macro_use]
extern crate log;

pub struct Tests {
    config: Config,
    tests: Vec<Test>,
}

impl Tests {
    fn new(config: Config, tests: Vec<Test>) -> Self {
        Self { config, tests }
    }

    /// Read a file and parse it to return tests
    ///
    /// # Errors
    /// Return an error if:
    /// * `file_path` is wrong
    /// * given file isn't a tests file
    pub fn from_file(file_path: &str) -> Result<Tests, String> {
        let file = std::fs::read_to_string(file_path).map_err(|err| err.to_string())?;
        let toml_parse: Value = toml::from_str(file.as_str()).map_err(|err| err.to_string())?;
        let tests = toml_parse
            .get("tests")
            .unwrap()
            .as_table()
            .unwrap()
            .clone()
            .into_iter()
            .map(|(i, value)| Test::from_value(i, &value))
            .collect::<Vec<Test>>();

        let config = Config::from_value(&toml_parse);
        Ok(Tests::new(config, tests).sort_from_file(&file))
    }

    fn sort_from_file(self, file: &str) -> Self {
        let tests_name = file
            .split('[')
            .filter(|f| f.contains("tests") && !f.contains("req"))
            .map(|v| v.trim())
            .filter_map(|v| v.strip_suffix(']'))
            .filter_map(|v| v.strip_prefix("tests."))
            .collect::<Vec<&str>>();

        let mut tests = self.tests.clone();
        let mut sort = Vec::new();
        for test_name in tests_name {
            for (n, i) in tests.iter().enumerate() {
                if i.name == test_name {
                    sort.push(i.clone());
                    tests.remove(n);
                    break;
                }
            }
        }

        sort.append(&mut tests);

        Self {
            config: self.config,
            tests: sort,
        }
    }

    pub async fn run(&mut self) {
        warn!("Start Test");
        let mut set_values = HashMap::new();
        let test_lenght = self.tests.len();
        warn!("Will run {test_lenght} test(s)");
        for (n, test) in self.tests.iter().enumerate() {
            warn!("[{}/{test_lenght}] Running {}", n + 1, test.name);
            let inst = Instant::now();
            for (key, val) in test.run(&self.config, &set_values).await {
                set_values.insert(key.trim().to_string(), val);
            }

            warn!(
                "[{}/{test_lenght}] Passed in {}ms",
                n + 1,
                inst.elapsed().as_millis()
            );
        }
    }
}
