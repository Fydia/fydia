use axum::{body::Bytes, extract::Extension};
use fydia_sql::{impls::user::SqlUser, sqlpool::DbConnection};
use fydia_struct::{
    instance::Instance,
    response::{FydiaResponse, FydiaResult},
    user::User,
};

use http::StatusCode;
use serde_json::Value;

use crate::handlers::get_json;

pub async fn create_user(body: Bytes, Extension(database): Extension<DbConnection>) -> FydiaResult {
    let string =
        String::from_utf8(body.to_vec()).map_err(|_| FydiaResponse::new_error("Utf-8 Body"))?;

    let json = serde_json::from_str::<Value>(&string)
        .map_err(|_| FydiaResponse::new_error("Body is not json"))?;

    let name = get_json("name".to_string(), &json)?;
    let email = get_json("email".to_string(), &json)?;
    let password = get_json("password".to_string(), &json)?;

    User::new(name, email, password, Instance::default())
        .map_err(FydiaResponse::new_error)?
        .insert_user(&database)
        .await
        .map(|_| FydiaResponse::new_ok("Register successfully"))
        .map_err(|error| {
            error!(error);
            FydiaResponse::new_error_custom_status(
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })
}
