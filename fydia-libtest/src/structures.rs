use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::{collections::HashMap, fmt::Display, process::exit};
use toml::Value;

#[derive(Debug, Clone)]
pub(crate) struct Headers(HashMap<String, String>);

impl Headers {
    pub fn from_value(value: &Value) -> Option<Self> {
        if let Some(headers) = value.get("headers") {
            if let Some(array) = headers.as_array() {
                let str_array = array
                    .iter()
                    .filter_map(|f| f.as_str())
                    .collect::<Vec<&str>>();

                let mut buf = HashMap::new();

                for i in 0..str_array.len() {
                    if i % 2 == 0 {
                        if let Some(headername_str) = str_array.get(i) {
                            if let Some(headervalue) = str_array.get(i + 1) {
                                buf.insert(
                                    (*headername_str).to_string(),
                                    (*headervalue).to_string(),
                                );
                            }
                        }
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
            let mut val = val.clone();

            val = if let Some(value) = setvalue.get(&val) {
                value.clone()
            } else {
                val
            };

            headermap.insert(
                HeaderName::from_bytes(key.as_bytes())
                    .map_err(|err| panic!("{key} is not a valid name: {}", err))
                    .expect("Header name is unvalid"),
                HeaderValue::from_bytes(val.as_bytes()).expect("Header value is unvalid"),
            );
        }

        headermap
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
            .expect("No req in a test")
            .as_array()
            .expect("Req is not an array")
            .iter()
            .map(Self::from_value_unique_element)
            .collect::<Vec<Self>>()
    }
    fn from_value_unique_element(value: &Value) -> Self {
        let r#type = value
            .get("type")
            .expect("No type request")
            .as_str()
            .expect("Type Request isn't a str")
            .to_string();

        let method = value
            .get("method")
            .map(|f| f.as_str().expect("method isn't a string"))
            .expect("Method isn't exists")
            .to_string();

        let url = value
            .get("url")
            .expect("No url in the test")
            .as_str()
            .expect("Url isn't a str")
            .to_string();
        let headers = Headers::from_value(value).expect("Header cannot be convert");
        let body = value
            .get("body")
            .expect("No body")
            .as_str()
            .expect("Body isn't a str")
            .to_string();
        let cmp_response =
            Response::from_value(value.get("response").expect("No response to compare"));

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
    pub fn from_value(value: &[Value]) -> Vec<Self> {
        value
            .iter()
            .map(|value| {
                let name = value
                    .get("name")
                    .expect("No name in a set_value")
                    .as_str()
                    .expect("Name isn't a str in a set_value");
                let json_path = value
                    .get("path")
                    .expect("No path in a set_value")
                    .as_str()
                    .expect("Path isn't a str in a set_value");

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
        let set_values = value
            .get("set_value")
            .map(|value| SetValue::from_value(value.as_array().expect("set_value isn't an array")));

        let body = value
            .get("body")
            .expect("No response body")
            .as_str()
            .expect("Response body isn't a str")
            .to_string();

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
    pub fn from_value(value: &Value) -> Self {
        let value = value.get("config").unwrap().as_table().unwrap();
        Self {
            url: value
                .get("url")
                .expect("No url in config table")
                .as_str()
                .expect("Url in config table isn't a str")
                .to_string(),
            port: value
                .get("port")
                .expect("No port in config table")
                .as_integer()
                .expect("port in config table isn't a integer") as u32,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "http://{}:{}", self.url, self.port)
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
        if self.request.r#type.to_uppercase() != "HTTP" {
            error!("Unknow type of test");
            panic!("");
        }

        let url = url_replace_set_value_name_with_value(
            format!("{}{}", config, self.request.url),
            setvalue,
        );
        let cmp_res = &self.request.cmp_response;
        let reqbuild = match self.request.method.to_uppercase().as_str() {
            "GET" => reqwest::Client::new().get(&url),
            "POST" => reqwest::Client::new().post(&url),
            "DELETE" => reqwest::Client::new().delete(&url),
            "PUT" => reqwest::Client::new().put(&url),
            _ => panic!("No method"),
        };
        info!(
            "(ReqType = {}) Send a {} request to {}",
            self.request.r#type, self.request.method, url
        );
        let res = reqbuild
            .headers(self.request.headers.to_headervalues(setvalue))
            .body(self.request.body.clone())
            .send()
            .await
            .unwrap();

        let res_status = res.status().as_u16();
        let res_body = res.text().await.expect("Body isn't a string");

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

                let value = value
                    .as_str()
                    .expect("Given path not target a string")
                    .to_string();

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

pub fn url_replace_set_value_name_with_value(
    url: String,
    hashmap: &HashMap<String, String>,
) -> String {
    let mut to_replace: Vec<(String, String)> = Vec::new();

    for i in url.split('{') {
        let mut value_name = i.split('}').collect::<Vec<&str>>()[0];
        value_name = value_name.trim();

        if let Some(value) = hashmap.get(value_name) {
            to_replace.push((format!("{{{}}}", value_name), value.clone()));
        };
    }

    let mut url = url;
    for replace in &to_replace {
        url = url.replace(&replace.0, &replace.1);
    }

    url
}
