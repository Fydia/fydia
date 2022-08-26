use fydia_struct::response::{FydiaResponse, MapError};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, InsertResult, IntoActiveModel};

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
) -> Result<InsertResult<A>, FydiaResponse<'a>> {
    T::insert(am).exec(executor).await.error_to_fydiaresponse()
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
) -> Result<(), FydiaResponse<'a>>
where
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
{
    T::update(am)
        .exec(executor)
        .await
        .map(|_| ())
        .error_to_fydiaresponse()
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
) -> Result<(), FydiaResponse<'a>> {
    T::delete(am)
        .exec(executor)
        .await
        .map(|_| ())
        .error_to_fydiaresponse()
}
