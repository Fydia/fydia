use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::response::FydiaResponse;
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
        FydiaResponse::new_error("Bad Token").update_response(&mut res);

        return Ok((state, res));
    };
    let database = &SqlPool::borrow_from(&state).get_pool();
    if let Some(user) = User::get_user_by_token(&token, database).await {
        FydiaResponse::new_ok_json(&user.server).update_response(&mut res);
    } else {
        FydiaResponse::new_error("Token error").update_response(&mut res);
    }

    Ok((state, res))
}
