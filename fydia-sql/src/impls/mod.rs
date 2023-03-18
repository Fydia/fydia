use fydia_struct::sqlerror::GenericSqlError;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, InsertResult, IntoActiveModel};
use shared::sea_orm;

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
    T::insert(am)
        .exec(executor)
        .await
        .map_err(|f| GenericSqlError::CannotInsert(f.to_string()))
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
    T::update(am)
        .exec(executor)
        .await
        .map(|_| ())
        .map_err(|f| GenericSqlError::CannotUpdate(f.to_string()))
}

/// Delete any model
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
/// * Model doesn't exist
pub async fn delete<'a, T: EntityTrait, A: ActiveModelTrait<Entity = T>>(
    am: A,
    executor: &DatabaseConnection,
) -> Result<(), GenericSqlError> {
    T::delete(am)
        .exec(executor)
        .await
        .map(|_| ())
        .map_err(|f| GenericSqlError::CannotDelete(f.to_string()))
}
