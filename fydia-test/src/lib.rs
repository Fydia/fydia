lazy_static::lazy_static! {
    static ref CONTEXT: Context = Context::default();
}

#[cfg(test)]
mod tests;

use fydia_utils::http::{HeaderName, HeaderValue};
use reqwest::Method;

#[derive(Debug)]
pub struct Context {
    url: String,
    port: u32,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            url: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

pub struct Test {
    method: Method,
    url: String,
    body: String,
    headers: Vec<(String, String)>,
    result: Option<TestResult>,
}

impl Test {
    pub fn new<T>(
        method: Method,
        url: T,
        body: T,
        headers: Vec<(String, String)>,
        result: Option<TestResult>,
    ) -> Self
    where
        T: Into<String>,
    {
        Self {
            method,
            url: url.into(),
            body: body.into(),
            headers,
            result,
        }
    }

    pub fn add_result(&mut self, result: TestResult) {
        self.result = Some(result);
    }

    ///
    /// # Errors
    ///
    pub async fn send(self) -> Result<(), String> {
        let ctx = &CONTEXT;
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}{}", ctx.url, ctx.port, self.url);
        let mut rq = match self.method.as_str().to_uppercase().as_str() {
            "POST" => client.post(url),
            "GET" => client.get(url),
            "DELETE" => client.delete(url),
            _ => return Err("Unknown method".to_string()),
        };

        for (key, value) in self.headers.iter() {
            let name = HeaderName::from_bytes(key.as_bytes()).unwrap();
            let value = HeaderValue::from_bytes(value.as_bytes()).unwrap();

            rq = rq.header(name, value);
        }

        rq = rq.body(self.body);

        let res = rq.send().await.unwrap();

        let statuscode = res.status().as_u16();
        let body = res.text().await.unwrap();
        let resulttest = self.result.unwrap();

        assert_eq!(statuscode, resulttest.status_code);

        if !resulttest.body.is_empty() {
            assert_eq!(body, resulttest.body);
        }

        Ok(())
    }
}

pub struct TestResult {
    status_code: u16,
    body: String,
}

impl TestResult {
    pub fn new<T: Into<String>>(status_code: u16, body: T) -> Self {
        Self {
            status_code,
            body: body.into(),
        }
    }
}

#[macro_export]
macro_rules! create_test {
    ($method:expr, $url:expr, $body:expr) => {
        $crate::Test::new($method, $url, $body, vec![], None)
    };

    ($method:expr, $url:expr, $body:expr, $headers:expr) => {
        $crate::Test::new($method, $url, $body, $headers, None)
    };
}

#[macro_export]
macro_rules! create_result {
    ($statuscode:expr, $body:expr, $test:expr) => {
        $test.add_result($crate::TestResult::new($statuscode, $body))
    };
}
