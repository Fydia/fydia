//! This module related to extractor of parameter

use serde::Deserialize;

/// Get the Url Parameter like ?token=SOMETOKEN
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub struct QsToken {
    pub token: Option<String>,
}
