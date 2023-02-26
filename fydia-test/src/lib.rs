#[cfg(test)]
mod tests;

use std::collections::HashMap;

use fydia_utils::{
    http::HeaderValue,
    serde::{Deserialize, Serialize},
    serde_json::Value,
};
use reqwest::header::HeaderName;

#[derive(Deserialize, Serialize, Default, Debug)]
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

    pub async fn run_tests(&self) {
        for i in self.tests.values() {
            i.run(&self.context).await
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct Context {
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

impl ToString for Body {
    fn to_string(&self) -> String {
        match self {
            Body::String(string) => string.to_owned(),
            Body::Json(json) => fydia_utils::serde_json::to_string(json).unwrap(),
        }
    }
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
    pub async fn run(&self, ctx: &Context) {
        match &self.test_type {
            StepType::Ws { body } => todo!(),
            StepType::Http {
                method,
                url,
                headers,
                body,
            } => {
                let client = reqwest::Client::new();
                let url = format!("http://{}:{}{}", ctx.url, ctx.port, url);
                let mut rq = match method.to_uppercase().as_str() {
                    "POST" => client.post(url),
                    "GET" => client.get(url),
                    "DELETE" => client.delete(url),
                    _ => panic!("Unknown method"),
                };

                if let Some(headers) = headers {
                    for (key, value) in headers.iter() {
                        let name = HeaderName::from_bytes(key.as_bytes()).unwrap();
                        let value = HeaderValue::from_bytes(value.as_bytes()).unwrap();

                        rq = rq.header(name, value);
                    }
                }

                rq = rq.body(body.to_string());

                let res = rq.send().await.unwrap();

                println!("{:?}", res.text().await);
            }
        }
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(crate = "fydia_utils::serde")]
#[serde(untagged)]
pub enum Test {
    OneStep(Step),
    Steps(Vec<Step>),
    Other(Value),

    #[default]
    None,
}

impl Test {
    pub async fn run(&self, ctx: &Context) {
        match self {
            Test::OneStep(test) => test.run(ctx).await,
            Test::Steps(tests) => {
                for test in tests {
                    test.run(ctx).await;
                }
            }
            Test::Other(v) => {
                let tos = fydia_utils::serde_json::to_string(v).unwrap();
                println!("{:#?}", v.as_array().unwrap());
                let a = fydia_utils::serde_json::from_str::<Vec<Step>>(&tos);
                println!("{:?}", a);
            }
            Test::None => panic!(),
        }
    }
}
