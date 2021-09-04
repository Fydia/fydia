use fydia_sql::{impls::user::SqlUser, sqlpool::SqlPool};
use fydia_struct::{instance::Instance, user::User};
use gotham::{
    handler::HandlerResult,
    helpers::http::response::create_empty_response,
    hyper::{body, Body},
    state::{FromState, State},
};
use reqwest::StatusCode;
use serde_json::Value;

pub async fn create_user(mut state: State) -> HandlerResult {
    let database = &SqlPool::borrow_from(&state).clone().get_pool();
    let mut res = create_empty_response(&state, StatusCode::OK);
    if let Ok(body_bytes) = body::to_bytes(Body::take_from(&mut state)).await {
        let body = body_bytes.to_vec();
        if let Ok(e) = String::from_utf8(body) {
            if let Ok(json) = serde_json::from_str::<Value>(&e) {
                let name = json.get("name");
                let email = json.get("email");
                let password = json.get("password");

                match (name, email, password) {
                    (Some(name), Some(email), Some(password)) => {
                        match (name.as_str(), email.as_str(), password.as_str()) {
                            (Some(name), Some(email), Some(password)) => {
                                if let Err(e) =
                                    User::new(name, email, password, Instance::default())
                                        .insert_user(database)
                                        .await
                                {
                                    error!(format!("{}", e));
                                    *res.body_mut() = "Error: Database".into();
                                    *res.status_mut() = StatusCode::BAD_REQUEST;
                                } else {
                                    *res.body_mut() = "Register successfully".into();
                                }
                            }
                            _ => {
                                *res.body_mut() = "Error: json".into();
                                *res.status_mut() = StatusCode::BAD_REQUEST;
                            }
                        }
                    }
                    _ => {
                        *res.body_mut() = "Error: json".into();
                        *res.status_mut() = StatusCode::BAD_REQUEST;
                    }
                }
            } else {
                *res.body_mut() = "Error: not json".into();
                *res.status_mut() = StatusCode::BAD_REQUEST;
            }
        } else {
            *res.body_mut() = "Error".into();
            *res.status_mut() = StatusCode::BAD_REQUEST;
        }
    }

    return Ok((state, res));
}
