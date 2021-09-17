use fydia_sql::impls::server::SqlServer;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::error::FydiaResponse;
use fydia_struct::server::Server;
use fydia_struct::user::{Token, User};
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{body, Body, HeaderMap, StatusCode};
use gotham::state::{FromState, State};
use serde_json::Value;

pub async fn create_server(mut state: State) -> HandlerResult {
    let headers = HeaderMap::take_from(&mut state);
    let mut res = create_response(&state, StatusCode::BAD_REQUEST, mime::TEXT_PLAIN_UTF_8, "");
    let token = if let Some(token) = Token::from_headervalue(&headers) {
        token
    } else {
        return Ok((state, res));
    };
    let database = &SqlPool::borrow_from(&state).get_pool();
    if let Ok(body_bytes) = body::to_bytes(Body::take_from(&mut state)).await {
        if let Ok(body) = String::from_utf8(body_bytes.to_vec()) {
            if let Some(mut user) = User::get_user_by_token(&token, database).await {
                if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
                    if let Some(name) = value.get("name") {
                        if let Some(name_str) = name.as_str() {
                            let mut server = Server::new();
                            server.name = name_str.to_string();
                            server.owner = user.id;
                            match server.insert_server(database).await {
                                Ok(_) => match server.join(&mut user, database).await {
                                    Ok(_) => FydiaResponse::new_ok(server.shortid)
                                        .update_response(&mut res),
                                    Err(e) => {
                                        FydiaResponse::new_error("Cannot join the server")
                                            .update_response(&mut res);
                                        error!(e);
                                    }
                                },
                                Err(e) => {
                                    FydiaResponse::new_error("Cannot join the server")
                                        .update_response(&mut res);
                                    error!(e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((state, res))
}
