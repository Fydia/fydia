use fydia_sql::{impls::token::SqlToken, sqlpool::SqlPool};
use fydia_struct::{pathextractor::UserExtractor, response::FydiaResponse, user::Token};
use gotham::{
    handler::HandlerResult,
    helpers::http::response::create_response,
    hyper::HeaderMap,
    state::{FromState, State},
};
use reqwest::StatusCode;

pub async fn create_direct_message(state: State) -> HandlerResult {
    let database = &SqlPool::borrow_from(&state).clone().get_pool();
    let mut res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, "");
    let target_user = UserExtractor::borrow_from(&state);
    let headers = HeaderMap::borrow_from(&state);
    if let Some(token) = Token::from_headervalue(headers) {
        if let Some(user) = token.get_user(database).await {
            println!("{:?}", user);
        } else {
            FydiaResponse::new_error("Bad Token").update_response(&mut res);
        }
    } else {
        FydiaResponse::new_error("No Token").update_response(&mut res);
    }
    info!(format!("{}", &target_user.id));
    return Ok((state, res));
}
