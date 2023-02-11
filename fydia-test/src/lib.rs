#[cfg(test)]
mod tests;

use std::collections::HashMap;

use fydia_utils::{
    serde::{Deserialize, Serialize},
    serde_json::Value,
};

#[derive(Deserialize, Serialize, Default)]
#[serde(crate = "fydia_utils::serde")]
struct LibTest {
    #[serde(skip)]
    client: reqwest::Client,
    #[serde(flatten)]
    context: Context,
    tests: HashMap<String, Test>,
}

impl LibTest {
    pub fn from_str(str: &str) -> Self {
        fydia_utils::serde_json::from_str::<LibTest>(str).unwrap()
    }

    pub fn run_tests(&self) {
        for i in self.tests.values() {
            i.run()
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
struct Context {
    url: String,
    port: u16,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            url: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
#[serde(untagged)]
pub enum Body {
    String(String),
    Json(fydia_utils::serde_json::Value),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    /// Message
    Ws { body: String },

    /// method, Body, headers to add
    Http {
        method: String,
        url: String,
        headers: Option<HashMap<String, String>>,
        body: Body,
    },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
#[serde(untagged)]
pub enum StepResultType {
    HttpString { status_code: u32, body: String },
    HttpValue { status_code: u32, body: Value },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct Step {
    #[serde(flatten)]
    test_type: StepType,
    result: StepResultType,
}

impl Step {
    pub fn run(&self) {
        match &self.test_type {
            StepType::Ws { body } => todo!(),
            StepType::Http {
                method,
                url,
                headers,
                body,
            } => todo!(),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "fydia_utils::serde")]
#[serde(untagged)]
pub enum Test {
    OneStep(Step),
    Steps(Vec<Step>),
    Other(Value),
    None,
}

impl Test {
    pub fn run(&self) {
        match self {
            Test::OneStep(test) => println!("{:?}", test),
            Test::Steps(tests) => println!("{:?}", tests),
            Test::Other(v) => {
                let tos = fydia_utils::serde_json::to_string(v).unwrap();
                println!("{:#?}", v.as_array().unwrap());
                let a = fydia_utils::serde_json::from_str::<Vec<Step>>(&tos);
                println!("{:?}", a);
            }
            _ => panic!(),
        }
    }
}

impl Default for Test {
    fn default() -> Self {
        Test::None
    }
}
