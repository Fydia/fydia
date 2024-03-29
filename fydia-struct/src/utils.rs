//! Usefull data structure
use fydia_utils::serde::{Deserialize, Serialize};
use thiserror::Error;

/// Enum to add a state of Id of a structure
#[derive(Debug, PartialEq, PartialOrd, Eq, Clone, Hash, Serialize, Deserialize)]
#[serde(crate = "fydia_utils::serde")]
#[serde(untagged)]
pub enum Id<T> {
    /// Id of structure
    Id(T),
    /// Unset when Id isn't set
    Unset,
}

impl<T> Id<T> {
    /// Return value of `Id` if `Id` is `Id(T)`
    ///
    /// # Errors
    /// Return an error if:
    /// * `Id` is unset
    pub fn get_id(self) -> Result<T, IdError> {
        if let Self::Id(id) = self {
            return Ok(id);
        }

        Err(IdError::IdUnset)
    }

    /// Return cloned value of `Id` if `Id` is `Id(T)`
    ///
    /// # Errors
    /// Returns an error if:
    /// * Id is unset
    pub fn get_id_fydiaresponse(self) -> Result<T, IdError> {
        if let Self::Id(id) = self {
            return Ok(id);
        }

        Err(IdError::IdUnset)
    }

    /// Return true if Id is `Id(T)`
    pub fn is_set(&self) -> bool {
        !matches!(self, Id::Unset)
    }

    /// Return true if Id is `Unset`
    pub fn is_not_set(&self) -> bool {
        !self.is_set()
    }

    /// Set an Id
    pub fn set(&mut self, id: T) {
        *self = Id::Id(id);
    }
}

impl<T: Clone> Id<T> {
    /// Return cloned value of `Id` if `Id` is `Id(T)`
    ///
    /// # Errors
    /// Returns an error if:
    /// * Id is unset
    pub fn get_id_cloned(&self) -> Result<T, IdError> {
        if let Self::Id(id) = &self {
            return Ok(id.clone());
        }

        Err(IdError::IdUnset)
    }
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `IdError` represents all errors of `Id`
pub enum IdError {
    #[error("Id is unset")]
    IdUnset,
}
