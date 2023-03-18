use crate::handlers::{basic::Database, get_json, get_json_value_from_body};

use fydia_sql::impls::user::SqlUser;
use fydia_struct::{instance::Instance, response::FydiaResult, user::User};

/// Create a new user
///
/// # Errors
/// This function will return an error if database is unreachable or if body
/// isn't valid
pub async fn create_user(Database(database): Database, body: String) -> FydiaResult {
    let json = get_json_value_from_body(&body)?;

    let name = get_json("name".to_string(), &json)?;
    let email = get_json("email".to_string(), &json)?;
    let password = get_json("password".to_string(), &json)?;

    User::new(name, email, password, Instance::default())?
        .insert(&database)
        .await?;

    "Register successfully".into()
}
