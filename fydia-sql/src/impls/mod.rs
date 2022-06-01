use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};

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
pub async fn insert<T: EntityTrait, A: ActiveModelTrait<Entity = T>>(
    am: A,
    executor: &DatabaseConnection,
) -> Result<(), String> {
    T::insert(am)
        .exec(executor)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// Update any model
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
/// * Model doesn't exist
pub async fn update<T: EntityTrait, A: ActiveModelTrait<Entity = T>>(
    am: A,
    executor: &DatabaseConnection,
) -> Result<(), String>
where
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
{
    T::update(am)
        .exec(executor)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
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
) -> Result<(), String> {
    T::delete(am)
        .exec(executor)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}
