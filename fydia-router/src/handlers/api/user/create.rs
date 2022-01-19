use axum::{body::Bytes, extract::Extension, response::IntoResponse};
use fydia_sql::{impls::user::SqlUser, sqlpool::DbConnection};
use fydia_struct::{instance::Instance, response::FydiaResponse, user::User};

use serde_json::Value;

pub async fn create_user(
    body: Bytes,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    if let Ok(e) = String::from_utf8(body.to_vec()) {
        if let Ok(json) = serde_json::from_str::<Value>(&e) {
            let name = json.get("name");
            let email = json.get("email");
            let password = json.get("password");

            match (name, email, password) {
                (Some(name), Some(email), Some(password)) => {
                    match (name.as_str(), email.as_str(), password.as_str()) {
                        (Some(name), Some(email), Some(password)) => {
                            if let Err(e) = User::new(name, email, password, Instance::default())
                                .insert_user(&database)
                                .await
                            {
                                error!(e);
                                return FydiaResponse::new_error("Database error");
                            }

                            return FydiaResponse::new_ok("Register successfully");
                        }
                        _ => return FydiaResponse::new_error("Json error"),
                    }
                }
                _ => return FydiaResponse::new_error("Json error"),
            }
        }

        return FydiaResponse::new_error("Body is not json");
    }

    FydiaResponse::new_error("Utf-8 Body")
}
