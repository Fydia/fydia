//! Modules of with Error num for `SqlError`
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(missing_docs)]
/// `GenericSqlError` represents all errors of sql generic function
pub enum GenericSqlError {
    #[error("{}", .0.error)]
    CannotInsert(GenericError),
    #[error("{}", .0.error)]
    CannotUpdate(GenericError),
    #[error("{}", .0.error)]
    CannotDelete(GenericError),
}

#[derive(Debug)]
/// `GenericError` contains usefull information for creation of error
pub struct GenericError {
    /// Setted column on sql request
    pub set_column: Vec<String>,
    /// Error message
    pub error: String,
}
