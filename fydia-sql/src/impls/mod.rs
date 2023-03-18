use fydia_struct::sqlerror::{GenericError, GenericSqlError};
use migration::DbErr;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, InsertResult, IntoActiveModel};
use shared::sea_orm;
use shared::sea_orm::Iterable;

pub mod basic_model;
pub mod channel;
pub mod direct_message;
pub mod emoji;
pub mod members;
pub mod message;
pub mod permission;
pub mod role;
pub mod server;
pub mod token;
pub mod user;

/// Insert any model in a table
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
/// * Table doesn't exist
/// * Model doesn't exist
pub async fn insert<'a, T: EntityTrait, A: ActiveModelTrait<Entity = T>>(
    am: A,
    executor: &DatabaseConnection,
) -> Result<InsertResult<A>, GenericSqlError> {
    let set_column = get_set_column::<T, A>(&am);
    T::insert(am)
        .exec(executor)
        .await
        .map_err(|f| GenericSqlError::CannotInsert(from_dberr_and_activemodel(set_column, &f)))
}

/// Update any model
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
/// * Model doesn't exist
pub async fn update<'a, T: EntityTrait, A: ActiveModelTrait<Entity = T>>(
    am: A,
    executor: &DatabaseConnection,
) -> Result<(), GenericSqlError>
where
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
{
    let set_column = get_set_column::<T, A>(&am);
    T::update(am)
        .exec(executor)
        .await
        .map(|_| ())
        .map_err(|f| GenericSqlError::CannotUpdate(from_dberr_and_activemodel(set_column, &f)))
}

/// Delete any model
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
/// * Model doesn't exist
pub async fn delete<T: EntityTrait, A: ActiveModelTrait<Entity = T>>(
    am: A,
    executor: &DatabaseConnection,
) -> Result<(), GenericSqlError> {
    let set_column = get_set_column::<T, A>(&am);
    T::delete(am)
        .exec(executor)
        .await
        .map(|_| ())
        .map_err(|f| GenericSqlError::CannotDelete(from_dberr_and_activemodel(set_column, &f)))
}

pub fn get_set_column<T: EntityTrait, A: ActiveModelTrait<Entity = T>>(am: &A) -> Vec<String> {
    let mut set_column = Vec::new();

    <T as EntityTrait>::Column::iter().for_each(|f| {
        if !am.is_not_set(f) {
            set_column.push(format!("{f:?}"));
        }
    });

    set_column
}

pub fn from_dberr_and_activemodel(set_column: Vec<String>, dberr: &DbErr) -> GenericError {
    GenericError {
        set_column,
        error: dberr.to_string(),
    }
}
