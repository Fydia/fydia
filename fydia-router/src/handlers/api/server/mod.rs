use fydia_sql::impls::server::{SqlServer, SqlServerId};
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::error::FydiaResponse;
use fydia_struct::server::{Server, ServerId};
use fydia_struct::user::{Token, User};
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{HeaderMap, StatusCode};
use gotham::state::{FromState, State};

pub mod channels;
pub mod create;
pub mod info;
pub mod join;
pub mod roles;

pub async fn get_server(state: State) -> HandlerResult {
    let database = &SqlPool::borrow_from(&state).get_pool();
    let headers = HeaderMap::borrow_from(&state);
    let mut res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, "");
    let serverid = ServerExtractor::borrow_from(&state);
    let mut servers: GetServer = GetServer { server: Vec::new() };
    if let Ok(getted_server) =
        Server::get_server_by_id(ServerId::new(serverid.serverid.clone()), database).await
    {
        let token = if let Some(token) = Token::from_headervalue(headers) {
            token
        } else {
            return Ok((state, res));
        };

        let server = User::get_user_by_token(&token, database).await;

        if let Some(e) = server {
            let a = e.server;
            for i in a.0 {
                if let Ok(server) = i.get_server(database).await {
                    servers.server.push(server);
                }
            }
        }

        FydiaResponse::new_ok_json(&getted_server).update_response(&mut res);
    }

    Ok((state, res))
}

use fydia_struct::pathextractor::ServerExtractor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetServer {
    pub server: Vec<Server>,
}
