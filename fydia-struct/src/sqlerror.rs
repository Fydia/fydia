//! Modules of with Error num for SqlError

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(missing_docs)]
/// `GenericSqlError` represents all errors of sql generic function
pub enum GenericSqlError {
    #[error("{0}")]
    CannotInsert(String),
    #[error("{0}")]
    CannotUpdate(String),
    #[error("{0}")]
    CannotDelete(String),
}
