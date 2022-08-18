use crate::handlers::{get_json, get_json_value_from_body};
use axum::{body::Bytes, extract::Extension};
use fydia_sql::{impls::user::SqlUser, sqlpool::DbConnection};
use fydia_struct::{
    instance::Instance,
    response::{FydiaResponse, FydiaResult},
    user::User,
};
use fydia_utils::http::StatusCode;

/// Create a new user
///
/// # Errors
/// This function will return an error if database is unreachable or if body
/// isn't valid
pub async fn create_user<'a>(
    body: Bytes,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let json = get_json_value_from_body(&body).map_err(FydiaResponse::StringError)?;

    let name = get_json("name".to_string(), &json)?;
    let email = get_json("email".to_string(), &json)?;
    let password = get_json("password".to_string(), &json)?;

    User::new(name, email, password, Instance::default())
        .map_err(FydiaResponse::StringError)?
        .insert(&database)
        .await
        .map(|_| FydiaResponse::Text("Register successfully"))
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            )
        })
}
