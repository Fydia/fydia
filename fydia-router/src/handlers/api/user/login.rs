use gotham::{
    handler::HandlerResult,
    helpers::http::response::create_response,
    hyper::{body, Body, StatusCode},
    state::{FromState, State},
};
use serde_json::value;

use fydia_sql::{impls::user::SqlUser, sqlpool::SqlPool};
use fydia_struct::user::User;

pub async fn user_login(mut state: State) -> HandlerResult {
    let database = &SqlPool::borrow_from(&state).clone().get_pool();
    let body = body::to_bytes(Body::take_from(&mut state))
        .await
        .expect("Error")
        .to_vec();

    let mut res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, "");
    if body.is_empty() {
        *res.body_mut() = "Bad body".into();
        *res.status_mut() = StatusCode::BAD_REQUEST;

        return Ok((state, res));
    }

    if let Ok(json) =
        serde_json::from_str::<value::Value>(String::from_utf8(body).unwrap().as_str())
    {
        match (json.get("email"), json.get("password")) {
            (Some(email), Some(password)) => match (email.as_str(), password.as_str()) {
                (Some(email), Some(password)) => {
                    let user = User::get_user_by_email_and_password(
                        email.to_string(),
                        password.to_string(),
                        database,
                    )
                    .await;

                    match user {
                        Some(user) => {
                            let token = user.unwrap().update_token(database).await.unwrap();
                            *res.body_mut() = token.into();
                        }
                        None => {
                            *res.status_mut() = StatusCode::BAD_REQUEST;
                            *res.body_mut() = "User not exists".into();
                        }
                    }
                }
                _ => {
                    *res.status_mut() = StatusCode::BAD_REQUEST;
                    *res.body_mut() = "Error on json".into();
                }
            },
            _ => {
                *res.status_mut() = StatusCode::BAD_REQUEST;
                *res.body_mut() = "Error on json".into();
            }
        }
    }

    Ok((state, res))
}
