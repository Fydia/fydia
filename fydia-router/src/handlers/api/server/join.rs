use fydia_sql::impls::server::SqlServer;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::pathextractor::ServerExtractor;
use fydia_struct::server::{Server, ServerId};
use fydia_struct::user::{Token, User};
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{HeaderMap, StatusCode};
use gotham::state::{FromState, State};

pub async fn join(mut state: State) -> HandlerResult {
    let headers = HeaderMap::take_from(&mut state);
    let mut res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, format!(""));
    let token = if let Some(token) = Token::from_headervalue(&headers) {
        token
    } else {
        return Ok((state, res));
    };
    let database = &SqlPool::borrow_from(&state).get_pool();
    let server = ServerExtractor::borrow_from(&state).serverid.clone();
    if let Some(mut user) = User::get_user_by_token(&token, database).await {
        if let Ok(mut server) = Server::get_server_by_id(ServerId::new(server), database).await {
            if user.server.is_join(ServerId::new(server.id.clone())) {
                *res.body_mut() = "Already join".into();
                *res.status_mut() = StatusCode::BAD_REQUEST;
            } else if let Err(error) = server.join(&mut user, database).await {
                *res.body_mut() = "Cannot join".into();
                *res.status_mut() = StatusCode::BAD_REQUEST;
                error!(error);
            };
        }
    }

    Ok((state, res))
}
