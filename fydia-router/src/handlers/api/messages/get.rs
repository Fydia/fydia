use crate::new_response;
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::impls::server::SqlServerId;
use fydia_sql::impls::token::SqlToken;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::ServerId;
use fydia_struct::user::Token;
use http::HeaderMap;

pub async fn get_message(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut res = new_response();
    let serverid = serverid.clone();
    let channelid = channelid.clone();
    let token = if let Some(token) = Token::from_headervalue(&headers) {
        token
    } else {
        FydiaResponse::new_error("No token").update_response(&mut res);
        return res;
    };

    if let Some(user) = token.get_user(&database).await {
        if user.servers.is_join(ServerId::new(serverid.clone())) {
            if let Some(serverid) = user.servers.get(serverid) {
                if let Ok(server) = serverid.get_server(&database).await {
                    if let Some(e) = server.channel.get_channel(channelid) {
                        if let Ok(message) = &e.get_messages(&database).await {
                            FydiaResponse::new_ok_json(&message).update_response(&mut res);
                        }
                    }
                }
            }
        }
    }

    res
}
