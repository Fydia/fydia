use fydia_sql::impls::server::SqlServer;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::server::Server;
use fydia_struct::user::{Token, User};
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{body, Body, HeaderMap, StatusCode};
use gotham::state::{FromState, State};
use serde_json::Value;

pub async fn create_server(mut state: State) -> HandlerResult {
    let headers = HeaderMap::take_from(&mut state);
    let mut res = create_response(
        &state,
        StatusCode::BAD_REQUEST,
        mime::TEXT_PLAIN_UTF_8,
        "BAD REQUEST".to_string(),
    );
    let token = if let Some(token) = Token::from_headervalue(&headers) {
        token
    } else {
        return Ok((state, res));
    };
    let database = &SqlPool::borrow_from(&state).get_pool();
    let body = String::from_utf8(
        body::to_bytes(Body::take_from(&mut state))
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    if let Some(user) = User::get_user_by_token(&token, database).await {
        let value = serde_json::from_str::<Value>(body.as_str()).unwrap();

        if let Some(name) = value.get("name") {
            let name_string = name.as_str().unwrap().to_string();
            let mut server = Server::new();
            server.name = name_string;
            server.owner = user.id;
            server.insert_server(database).await;
            server.join(user, database).await;

            *res.status_mut() = StatusCode::OK;
            *res.body_mut() = server.shortid.into();
        }
    }

    Ok((state, res))
}
