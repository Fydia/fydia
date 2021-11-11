use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QsToken {
    pub token: Option<String>,
}
