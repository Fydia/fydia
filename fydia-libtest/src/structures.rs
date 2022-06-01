use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::{collections::HashMap, process::exit};
use toml::Value;

#[derive(Debug, Clone)]
pub(crate) struct Headers(HashMap<String, String>);

impl Headers {
    pub fn from_value(value: &Value) -> Option<Self> {
        if let Some(headers) = value.get("headers") {
            if let Some(array) = headers.as_array() {
                let mut buf = HashMap::new();
                let (mut left, mut right) = (None, None);
                for (n, i) in array
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(|str| str.to_string())
                    .enumerate()
                {
                    if n % 2 == 0 {
                        left = Some(i);
                        if n == 0 {
                            continue;
                        };

                        if left.is_none() || right.is_none() {
                            continue;
                        }

                        buf.insert(left.unwrap(), right.unwrap());
                        (left, right) = (None, None);
                    } else {
                        right = Some(i);
                    }
                }

                return Some(Self(buf));
            }
        }

        None
    }

    pub fn to_headervalues(&self, setvalue: &HashMap<String, String>) -> HeaderMap {
        let mut headermap = HeaderMap::new();

        for (key, val) in &self.0 {
            let val = if let Some(val) = setvalue.get(key) {
                val
            } else {
                val
            };

            headermap.insert(
                HeaderName::from_bytes(key.as_bytes())
                    .map_err(|err| panic!("{key} is not a valid name: {}", err.to_string()))
                    .unwrap(),
                HeaderValue::from_bytes(val.as_bytes()).unwrap(),
            );
        }

        return headermap;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Request {
    r#type: String,
    method: String,
    url: String,
    headers: Headers,
    body: String,
    cmp_response: Response,
}

impl Request {
    pub fn from_value(value: &Value) -> Vec<Self> {
        value
            .get("req")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|test| Self::from_value_unique_element(test))
            .collect::<Vec<Self>>()
    }
    fn from_value_unique_element(value: &Value) -> Self {
        let r#type = value.get("type").unwrap().as_str().unwrap().to_string();
        let method = value
            .get("method")
            .map(|f| f.as_str().unwrap())
            .unwrap()
            .to_string();
        let url = value.get("url").unwrap().as_str().unwrap().to_string();
        let headers = Headers::from_value(value).unwrap();
        let body = value.get("body").unwrap().as_str().unwrap().to_string();
        let cmp_response = Response::from_value(&value.get("response").unwrap());

        Self {
            r#type,
            method,
            url,
            headers,
            body,
            cmp_response,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SetValue {
    name: String,
    json_path: String,
}

impl SetValue {
    pub fn from_value(value: &Vec<Value>) -> Vec<Self> {
        value
            .iter()
            .map(|value| {
                let name = value.get("name").unwrap().as_str().unwrap();
                let json_path = value.get("path").unwrap().as_str().unwrap();

                Self {
                    name: name.to_string(),
                    json_path: json_path.to_string(),
                }
            })
            .collect::<Vec<Self>>()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Response {
    statuscode: u16,
    body: String,
    set_values: Option<Vec<SetValue>>,
}

impl Response {
    pub fn from_value(value: &Value) -> Self {
        let set_values = if let Some(set_values) = value.get("set_value") {
            Some(SetValue::from_value(set_values.as_array().unwrap()))
        } else {
            None
        };

        let body = value.get("body").unwrap().as_str().unwrap().to_string();

        let statuscode = if let Some(toml_value) = value.get("statuscode") {
            toml_value.as_integer().unwrap() as u16
        } else {
            200
        };

        Self {
            statuscode,
            body,
            set_values,
        }
    }
}

pub(crate) struct Config {
    url: String,
    port: u32,
}

impl Config {
    pub fn to_string(&self) -> String {
        format!("http://{}:{}", self.url, self.port)
    }

    pub fn from_value(value: Value) -> Self {
        let value = value.get("config").unwrap().as_table().unwrap();
        Self {
            url: value.get("url").unwrap().as_str().unwrap().to_string(),
            port: value.get("port").unwrap().as_integer().unwrap() as u32,
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Test {
    pub name: String,
    request: Request,
}

impl Test {
    pub fn from_value(name: String, value: &Value) -> Self {
        Self {
            name,
            request: Request::from_value(value)[0].clone(),
        }
    }

    pub async fn run(
        &self,
        config: &Config,
        setvalue: &HashMap<String, String>,
    ) -> Vec<(String, String)> {
        let url = format!("{}{}", config.to_string(), self.request.url);
        let cmp_res = &self.request.cmp_response;

        let res = match self.request.method.to_uppercase().as_str() {
            "GET" => reqwest::Client::new()
                .get(&url)
                .headers(self.request.headers.to_headervalues(setvalue))
                .body(self.request.body.clone())
                .send()
                .await
                .unwrap(),

            "POST" => reqwest::Client::new()
                .post(&url)
                .headers(self.request.headers.to_headervalues(setvalue))
                .body(self.request.body.clone())
                .send()
                .await
                .unwrap(),

            "DELETE" => reqwest::Client::new()
                .delete(&url)
                .headers(self.request.headers.to_headervalues(setvalue))
                .body(self.request.body.clone())
                .send()
                .await
                .unwrap(),

            _ => panic!("No method"),
        };
        let res_status = res.status().as_u16();
        let res_body = res.text().await.unwrap();

        if cmp_res.statuscode != res_status {
            error!(
                "({} {}) => StatusCode is not the same ({} != {})",
                self.request.method, &url, cmp_res.statuscode, res_status
            );
            error!("{res_body}");
            panic!("");
        }

        if !res_body.starts_with(&cmp_res.body) {
            error!(
                "({} {}) => Response body not starts with the given body",
                self.request.method, &url
            );
            panic!("");
        }

        if cmp_res.set_values.is_none() {
            return Vec::new();
        }

        let mut result = Vec::new();
        if let Ok(json) =
            serde_json::from_str::<serde_json::Value>(res_body.as_str()).map_err(|err| {
                error!("{err}");
                err
            })
        {
            for set_value in cmp_res.set_values.as_ref().unwrap() {
                let name = set_value.name.clone();
                let path = &set_value.json_path;
                let paths = path.split('/').collect::<Vec<&str>>();
                let mut value: &serde_json::Value = &json;
                for i in paths {
                    if let Some(getvalue) = value.get(i) {
                        value = getvalue;
                        continue;
                    }

                    error!("Cannot get {}", path);
                    exit(1);
                }

                let value = value.as_str().unwrap().to_string();

                result.push((name, value));
            }

            return result;
        }

        error!(
            "({} {}) => Response body isn't json",
            self.request.method, &url
        );
        exit(1);
    }
}
