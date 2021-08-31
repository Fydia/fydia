use serde::Deserialize;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct QsToken {
    pub token: Option<String>,
}
