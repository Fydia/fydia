use sea_orm::{
    sea_query::IntoCondition, ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    FromQueryResult, IntoActiveModel, IntoSimpleExpr, Order, PaginatorTrait, QueryFilter,
    QueryOrder, Select,
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

pub fn get_select_with_filter<T: EntityTrait, S: IntoCondition>(
    _: T,
    simplexprs: Vec<S>,
) -> Select<T> {
    let mut select = T::find();
    for i in simplexprs {
        select = select.filter(i);
    }

    select
}

pub async fn get_one<T: EntityTrait, S: IntoCondition>(
    entity: T,
    simplexprs: Vec<S>,
    executor: &DatabaseConnection,
) -> Result<T::Model, String> {
    let select = get_select_with_filter(entity, simplexprs);

    select
        .one(executor)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "This is not exists in DB".to_string())
}

pub async fn get_all<T: EntityTrait, S: IntoCondition>(
    entity: T,
    simplexprs: Vec<S>,
    executor: &DatabaseConnection,
) -> Result<Vec<T::Model>, String> {
    let select = get_select_with_filter(entity, simplexprs);
    select
        .all(executor)
        .await
        .map_err(|error| error.to_string())
}

pub async fn get_all_with_limit<
    'db,
    C: ConnectionTrait,
    T: EntityTrait<Model = M>,
    S: IntoCondition,
    M: FromQueryResult + Sized + Send + Sync + 'db,
>(
    entity: T,
    simplexprs: Vec<S>,
    limit: i32,
    executor: &C,
) -> Result<Vec<T::Model>, String> {
    let select = get_select_with_filter(entity, simplexprs);

    let page = select.paginate(executor, limit as usize);

    let page = page.fetch().await.map_err(|error| error.to_string())?;

    Ok(page)
}

pub async fn get_all_with_limit_with_order<
    'db,
    C: ConnectionTrait,
    T: EntityTrait<Model = M>,
    S: IntoCondition,
    E: IntoSimpleExpr,
    M: FromQueryResult + Sized + Send + Sync + 'db,
>(
    entity: T,
    simplexprs: Vec<S>,
    (column, order): (E, Order),
    limit: i32,
    executor: &C,
) -> Result<Vec<T::Model>, String> {
    let select = get_select_with_filter(entity, simplexprs).order_by(column, order);

    let page = select.paginate(executor, limit as usize);

    let page = page.fetch().await.map_err(|error| error.to_string())?;

    Ok(page)
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
