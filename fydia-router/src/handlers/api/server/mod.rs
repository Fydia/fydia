use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::impls::server::{SqlServer, SqlServerId};
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::{Server, ServerId};
use fydia_struct::user::{Token, User};
use http::HeaderMap;

pub mod channels;
pub mod create;
pub mod info;
pub mod join;
pub mod picture;
pub mod roles;

pub async fn get_server(
    headers: HeaderMap,
    Path(serverid): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();
    let mut servers: GetServer = GetServer { server: Vec::new() };
    if let Ok(getted_server) =
        Server::get_server_by_id(ServerId::new(serverid.clone()), &database).await
    {
        let token = if let Some(token) = Token::from_headervalue(&headers) {
            token
        } else {
            return res;
        };

        let server = User::get_user_by_token(&token, &database).await;

        if let Some(user) = server {
            for i in user.servers.0 {
                if let Ok(server) = i.get_server(&database).await {
                    servers.server.push(server);
                }
            }
        }

        FydiaResponse::new_ok_json(&getted_server).update_response(&mut res);
    }

    res
}

use serde::{Deserialize, Serialize};

use crate::new_response;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetServer {
    pub server: Vec<Server>,
}
