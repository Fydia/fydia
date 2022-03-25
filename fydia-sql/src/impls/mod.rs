use sea_orm::{
    sea_query::IntoCondition, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter,
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

pub async fn get_all<T: EntityTrait, S: IntoCondition>(
    _: T,
    simplexprs: Vec<S>,
    executor: &DatabaseConnection,
) -> Result<Vec<T::Model>, String> {
    let mut select = T::find();
    for i in simplexprs {
        select = select.filter(i);
    }

    select
        .all(executor)
        .await
        .map_err(|error| error.to_string())
}

pub async fn get_one<T: EntityTrait, S: IntoCondition>(
    _: T,
    simplexprs: Vec<S>,
    executor: &DatabaseConnection,
) -> Result<T::Model, String> {
    let mut select = T::find();
    for i in simplexprs {
        select = select.filter(i);
    }

    select
        .one(executor)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "This is not exists in DB".to_string())
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
