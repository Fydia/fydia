use fydia_utils::http::{HeaderName, HeaderValue};
use reqwest::Method;

#[cfg(test)]
mod tests;

pub struct TestExpect {
    statuscode: Option<u16>,
    body: Option<&'static str>,
}

pub struct TestRunnable<'a> {
    ctx: &'a TestContext,
    method: Method,
    path: &'a str,
    body: Option<&'static str>,
    headers: Option<Vec<(&'a str, &'a str)>>,
    expect: Option<TestExpect>,
}

impl<'a> TestRunnable<'a> {
    #[must_use]
    pub fn body(mut self, body: &'static str) -> Self {
        self.body = Some(body);

        self
    }

    #[must_use]
    pub fn header(mut self, name: &'a str, value: &'a str) -> Self {
        if let Some(headers) = &mut self.headers {
            headers.push((name, value));

            return self;
        }

        self.headers = Some(vec![(name, value)]);

        self
    }

    #[must_use]
    pub fn expect_statuscode(mut self, statuscode: u16) -> Self {
        if let Some(expect) = &mut self.expect {
            expect.statuscode = Some(statuscode);
            return self;
        }

        self.expect = Some(TestExpect {
            statuscode: Some(statuscode),
            body: None,
        });

        self
    }

    #[must_use]
    pub fn expect_body(mut self, body: &'static str) -> Self {
        if let Some(expect) = &mut self.expect {
            expect.body = Some(body);
            return self;
        }

        self.expect = Some(TestExpect {
            statuscode: None,
            body: Some(body),
        });

        self
    }

    /// # Errors
    /// Return error if expected body or statuscode is different
    /// # Panics
    /// Panics if headername or headervalue is incorrect
    pub async fn send(self) -> Result<(), String> {
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}{}", self.ctx.url, self.ctx.port, self.path);
        let mut rq = match self.method.as_str().to_uppercase().as_str() {
            "POST" => client.post(url),
            "GET" => client.get(url),
            "DELETE" => client.delete(url),
            _ => return Err("Unknown method".to_string()),
        };

        if let Some(headers) = self.headers {
            for (key, value) in &headers {
                let name = HeaderName::from_bytes(key.as_bytes()).unwrap();
                let value = HeaderValue::from_bytes(value.as_bytes()).unwrap();

                rq = rq.header(name, value);
            }
        }

        if let Some(body) = self.body {
            rq = rq.body(body);
        }

        let res = rq.send().await.unwrap();

        let statuscode = res.status().as_u16();
        let body = res.text().await.unwrap();

        if let Some(expected) = self.expect {
            if let Some(expected_statuscode) = expected.statuscode {
                assert_eq!(statuscode, expected_statuscode);
            }

            if let Some(expected_body) = expected.body {
                assert_eq!(body, expected_body);
            }
        }

        Ok(())
    }
}

pub struct TestContext {
    url: &'static str,
    port: u16,
}

impl TestContext {
    #[must_use]
    pub const fn new(url: &'static str, port: u16) -> Self {
        Self { url, port }
    }

    #[must_use]
    pub fn get<'a>(&'a self, path: &'a str) -> TestRunnable<'a> {
        TestRunnable {
            ctx: self,
            method: Method::GET,
            path,
            body: None,
            headers: None,
            expect: None,
        }
    }

    #[must_use]
    pub fn post<'a>(&'a self, path: &'a str) -> TestRunnable<'a> {
        TestRunnable {
            ctx: self,
            method: Method::POST,
            path,
            body: None,
            headers: None,
            expect: None,
        }
    }

    #[must_use]
    pub fn delete<'a>(&'a self, path: &'a str) -> TestRunnable<'a> {
        TestRunnable {
            ctx: self,
            method: Method::DELETE,
            path,
            body: None,
            headers: None,
            expect: None,
        }
    }
}
