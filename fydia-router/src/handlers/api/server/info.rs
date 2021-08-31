use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::user::{Token, User};
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{HeaderMap, StatusCode};
use gotham::state::{FromState, State};

pub async fn get_server_of_user(mut state: State) -> HandlerResult {
    let headers = HeaderMap::take_from(&mut state);
    let mut res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, "");
    let token = if let Some(token) = Token::from_headervalue(&headers) {
        token
    } else {
        return Ok((state, res));
    };
    let database = &SqlPool::borrow_from(&state).get_pool();
    let user = User::get_user_by_token(&token, database).await.unwrap();
    *res.body_mut() = serde_json::to_string(&user.server).unwrap().into();
    Ok((state, res))
}
