//! This module related to extractor of parameter

use fydia_utils::serde::Deserialize;

/// Get the Url Parameter like ?token=SOMETOKEN
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct QsToken {
    pub token: Option<String>,
}
