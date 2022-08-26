use crate::handlers::{get_json, get_json_value_from_body};
use axum::{body::Bytes, extract::Extension};
use fydia_sql::{impls::user::SqlUser, sqlpool::DbConnection};
use fydia_struct::{
    instance::Instance,
    response::{FydiaResult, IntoFydia, MapError},
    user::User,
};

/// Create a new user
///
/// # Errors
/// This function will return an error if database is unreachable or if body
/// isn't valid
pub async fn create_user<'a>(
    body: Bytes,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let json = get_json_value_from_body(&body)?;

    let name = get_json("name".to_string(), &json)?;
    let email = get_json("email".to_string(), &json)?;
    let password = get_json("password".to_string(), &json)?;

    User::new(name, email, password, Instance::default())
        .error_to_fydiaresponse()?
        .insert(&database)
        .await
        .map(|_| "Register successfully".into_ok())
}
