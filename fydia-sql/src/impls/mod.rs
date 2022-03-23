use sea_orm::{
    sea_query::SimpleExpr, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
};

pub mod channel;
pub mod emoji;
pub mod members;
pub mod message;
pub mod permission;
pub mod role;
pub mod server;
pub mod token;
pub mod user;

#[async_trait::async_trait]
trait DefaultSql {
    type SelfModel;
    type ActiveModel;
    type ResultData;

    async fn get(
        expr: SimpleExpr,
        executor: &DatabaseConnection,
    ) -> Result<Self::ResultData, String>;

    async fn get_by_id(executor: &DatabaseConnection) -> Result<Self::SelfModel, String>;
    ///
    async fn insert_model(executor: &DatabaseConnection) -> Result<Self::SelfModel, String>;
    async fn insert(
        am: Self::ActiveModel,
        executor: &DatabaseConnection,
    ) -> Result<Self::SelfModel, String>;
}

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
